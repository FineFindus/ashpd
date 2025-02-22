use std::fs::File;

use adw::prelude::*;
use ashpd::{desktop::email::EmailRequest, WindowIdentifier};
use gtk::{gio, glib, subclass::prelude::*};

use crate::{
    portals::{is_empty, split_comma},
    widgets::{NotificationKind, PortalPage, PortalPageExt, PortalPageImpl},
};

mod imp {
    use adw::subclass::prelude::*;

    use super::*;
    use crate::widgets::RemovableRow;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/email.ui")]
    pub struct EmailPage {
        #[template_child]
        pub subject: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub body: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub addresses: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub cc_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub bcc_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub attachments_listbox: TemplateChild<gtk::ListBox>,
        pub model: gio::ListStore,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EmailPage {
        const NAME: &'static str = "EmailPage";
        type Type = super::EmailPage;
        type ParentType = PortalPage;

        fn new() -> Self {
            Self {
                subject: TemplateChild::default(),
                body: TemplateChild::default(),
                addresses: TemplateChild::default(),
                cc_entry: TemplateChild::default(),
                bcc_entry: TemplateChild::default(),
                attachments_listbox: TemplateChild::default(),
                model: gio::ListStore::new(gio::File::static_type()),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async(
                "email.compose",
                None,
                move |page, _action, _target| async move {
                    page.compose_mail().await;
                },
            );
            klass.install_action_async(
                "email.attach",
                None,
                move |page, _action, _target| async move {
                    page.attach().await;
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for EmailPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.attachments_listbox.bind_model(
                Some(&self.model),
                glib::clone!(@strong self.model as model => move |obj| {
                    let attachment = obj.downcast_ref::<gio::File>().unwrap();
                    let display_name = attachment
                        .query_info(
                            &gio::FILE_ATTRIBUTE_STANDARD_DISPLAY_NAME,
                            gio::FileQueryInfoFlags::NONE,
                            gio::Cancellable::NONE,
                        )
                        .unwrap()
                        .display_name();

                    let row = RemovableRow::default();
                    row.connect_removed(glib::clone!(@strong model => move |row| {
                        let index = row.index();
                        model.remove(index as u32);
                    }));
                    row.set_title(&display_name);
                    row.set_subtitle(&attachment.uri());
                    row.upcast()
                }),
            );

            self.model
                .connect_items_changed(glib::clone!(@weak obj => move |model, _, _, _| {
                    obj.imp().attachments_listbox.set_visible(model.n_items() > 0);
                }));
        }
    }
    impl WidgetImpl for EmailPage {}
    impl BinImpl for EmailPage {}
    impl PortalPageImpl for EmailPage {}
}

glib::wrapper! {
    pub struct EmailPage(ObjectSubclass<imp::EmailPage>)
        @extends gtk::Widget, adw::Bin, PortalPage;
}

impl EmailPage {
    async fn compose_mail(&self) {
        let imp = self.imp();

        let subject = is_empty(imp.subject.text());
        let body = is_empty(imp.body.text());
        let addresses = is_empty(imp.addresses.text()).map(split_comma);
        let bcc = is_empty(imp.bcc_entry.text()).map(split_comma);
        let cc = is_empty(imp.cc_entry.text()).map(split_comma);
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let mut request = EmailRequest::default()
            .identifier(identifier)
            .subject(subject.as_deref())
            .addresses::<Vec<_>, String>(addresses)
            .cc::<Vec<_>, String>(cc)
            .bcc::<Vec<_>, String>(bcc)
            .body(body.as_deref());
        let attachments = self.attachments();
        if !attachments.is_empty() {
            // TODO: add a request.set_attachments helper method
            attachments.iter().for_each(|attachment| {
                request.add_attachment(attachment);
            });
        }
        match request.build().await {
            Ok(_) => {
                self.send_notification(
                    "Compose an email request was successful",
                    NotificationKind::Success,
                );
            }
            Err(_err) => self.send_notification(
                "Request to compose an email failed",
                NotificationKind::Error,
            ),
        }
    }

    pub fn attachments(&self) -> Vec<File> {
        self.imp()
            .model
            .snapshot()
            .into_iter()
            .map(|obj| {
                let file = obj.downcast_ref::<gio::File>().unwrap();
                File::open(file.path().unwrap()).unwrap()
            })
            .collect::<Vec<File>>()
    }

    pub async fn attach(&self) {
        let dialog = gtk::FileChooserNative::builder()
            .select_multiple(true)
            .modal(true)
            .transient_for(self.native().and_downcast_ref::<gtk::Window>().unwrap())
            .build();
        if dialog.run_future().await == gtk::ResponseType::Accept {
            let files = dialog.files();
            for file in files.into_iter() {
                let file = file.ok().and_downcast::<gio::File>().unwrap();
                self.imp().model.append(&file);
            }
        }
    }
}
