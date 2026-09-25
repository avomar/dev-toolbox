use {
    crate::{
        Message,
        app::AppModel,
        class::text_input_style,
        components::{copy_button, page_header, status_bar},
        fl,
        utility_pages::UtilityPage,
    },
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, clipboard,
            widget::{column, row},
        },
        widget::{self, Id, text_input},
    },
    rand::RngCore,
    uuid::{ContextV1, NoContext, Timestamp, Uuid},
};

const OUTPUT_TEXT_ID: &str = "uuid-output";
const VERSIONS: [&str; 5] = ["UUIDv4", "UUIDv5", "UUIDv6", "UUIDv7", "UUIDv8"];
const MAX_V6_UNIX_MILLIS: u64 = ((1_u64 << 60) - 1 - 0x01b21dd213814000) / 10_000;
const MAX_V7_UNIX_MILLIS: u64 = (1_u64 << 48) - 1;

#[derive(Debug, Clone)]
pub enum UuidGeneratorMessage {
    VersionChanged(usize),
    NamespaceChanged(usize),
    CustomNamespaceChanged(String),
    NameChanged(String),
    V6TimestampChanged(String),
    V6NodeChanged(String),
    SetV6CurrentTime,
    RandomizeV6Node,
    V7TimestampChanged(String),
    SetV7CurrentTimeAndGenerate,
    V8DataChanged(String),
    Generate,
    CopyOutput,
    SelectAllOutput,
    ReadOnlyOutput,
}

pub(crate) struct UuidGeneratorPage {
    version: usize,
    namespace: usize,
    custom_namespace: String,
    name: String,
    v6_timestamp: String,
    v6_node: String,
    v6_initialized: bool,
    v6_context: ContextV1,
    v7_timestamp: String,
    v7_initialized: bool,
    v8_data: String,
    output: String,
    status: &'static str,
}

impl Default for UuidGeneratorPage {
    fn default() -> Self {
        Self {
            version: 0,
            namespace: 0,
            custom_namespace: String::new(),
            name: String::new(),
            v6_timestamp: String::new(),
            v6_node: String::new(),
            v6_initialized: false,
            v6_context: ContextV1::new_random(),
            v7_timestamp: String::new(),
            v7_initialized: false,
            v8_data: String::new(),
            output: String::new(),
            status: "ok",
        }
    }
}

impl UtilityPage for UuidGeneratorPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let options_header: Element<'_, Message> = widget::text::title4(fl!("options"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();
        let version_option = widget::settings::item(
            fl!("uuid-generator", "version"),
            widget::dropdown(VERSIONS.to_vec(), Some(self.version), |selection| {
                Message::UuidGeneratorMessage(UuidGeneratorMessage::VersionChanged(selection))
            }),
        );

        let mut version_options = column![].spacing(space_s);

        match self.version {
            1 => {
                let namespaces = vec![
                    "DNS".to_string(),
                    "OID".to_string(),
                    "URL".to_string(),
                    "X.500".to_string(),
                    fl!("uuid-generator", "custom"),
                ];
                version_options = version_options.push(widget::settings::item(
                    fl!("uuid-generator", "namespace"),
                    widget::dropdown(namespaces, Some(self.namespace), |selection| {
                        Message::UuidGeneratorMessage(UuidGeneratorMessage::NamespaceChanged(
                            selection,
                        ))
                    }),
                ));
                if self.namespace == 4 {
                    version_options = version_options.push(widget::settings::item(
                        fl!("uuid-generator", "custom-namespace"),
                        widget::text_input("", &self.custom_namespace)
                            .style(text_input_style())
                            .on_input(|value| {
                                Message::UuidGeneratorMessage(
                                    UuidGeneratorMessage::CustomNamespaceChanged(value),
                                )
                            }),
                    ));
                }
                version_options = version_options.push(widget::settings::item(
                    fl!("uuid-generator", "name"),
                    widget::text_input("", &self.name)
                        .style(text_input_style())
                        .on_input(|value| {
                            Message::UuidGeneratorMessage(UuidGeneratorMessage::NameChanged(value))
                        }),
                ));
            }
            2 => {
                version_options = version_options.push(widget::settings::item(
                    fl!("uuid-generator", "timestamp-milliseconds"),
                    row![
                        widget::text_input("", &self.v6_timestamp)
                            .style(text_input_style())
                            .width(Length::Fill)
                            .on_input(|value| {
                                Message::UuidGeneratorMessage(
                                    UuidGeneratorMessage::V6TimestampChanged(value),
                                )
                            }),
                        widget::button::text(fl!("uuid-generator", "now")).on_press(
                            Message::UuidGeneratorMessage(UuidGeneratorMessage::SetV6CurrentTime,),
                        ),
                    ]
                    .spacing(space_s),
                ));
                version_options = version_options.push(widget::settings::item(
                    fl!("uuid-generator", "node-id"),
                    row![
                        widget::text_input("", &self.v6_node)
                            .style(text_input_style())
                            .width(Length::Fill)
                            .on_input(|value| {
                                Message::UuidGeneratorMessage(UuidGeneratorMessage::V6NodeChanged(
                                    value,
                                ))
                            }),
                        widget::button::text(fl!("randomize")).on_press(
                            Message::UuidGeneratorMessage(UuidGeneratorMessage::RandomizeV6Node),
                        ),
                    ]
                    .spacing(space_s),
                ));
            }
            3 => {
                version_options = version_options.push(widget::settings::item(
                    fl!("uuid-generator", "timestamp-milliseconds"),
                    row![
                        widget::text_input("", &self.v7_timestamp)
                            .style(text_input_style())
                            .width(Length::Fill)
                            .on_input(|value| {
                                Message::UuidGeneratorMessage(
                                    UuidGeneratorMessage::V7TimestampChanged(value),
                                )
                            }),
                        widget::button::text(fl!("uuid-generator", "now")).on_press(
                            Message::UuidGeneratorMessage(
                                UuidGeneratorMessage::SetV7CurrentTimeAndGenerate,
                            ),
                        ),
                    ]
                    .spacing(space_s),
                ));
            }
            4 => {
                version_options = version_options.push(widget::settings::item(
                    fl!("uuid-generator", "custom-data"),
                    widget::text_input("", &self.v8_data)
                        .style(text_input_style())
                        .on_input(|value| {
                            Message::UuidGeneratorMessage(UuidGeneratorMessage::V8DataChanged(
                                value,
                            ))
                        }),
                ));
            }
            _ => {}
        }

        let output_header: Element<'_, Message> = row![
            widget::text::title4(fl!("output"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::button::text(fl!("generate")).on_press(Message::UuidGeneratorMessage(
                UuidGeneratorMessage::Generate
            )),
            copy_button(Message::UuidGeneratorMessage(
                UuidGeneratorMessage::CopyOutput,
            )),
        ]
        .align_y(Alignment::Center)
        .spacing(space_s)
        .into();

        let output = widget::text_input("", &self.output)
            .id(Id::new(OUTPUT_TEXT_ID))
            .style(text_input_style())
            .editing(false)
            .on_input(|_| Message::UuidGeneratorMessage(UuidGeneratorMessage::ReadOnlyOutput))
            .select_on_focus(true)
            .on_focus(Message::UuidGeneratorMessage(
                UuidGeneratorMessage::SelectAllOutput,
            ));

        column![
            page_header(Self::PAGE_NAME),
            options_header,
            version_option,
            version_options,
        ]
        .spacing(space_s)
        .height(Length::Fill)
        .push(output_header)
        .push(output)
        .push(widget::space().height(Length::Fill))
        .push(status_bar(self.status, Self::PAGE_NAME))
        .into()
    }

    fn handle_message(
        &mut self,
        message: Message,
    ) -> Task<cosmic::Action<<AppModel as Application>::Message>> {
        if let Message::UuidGeneratorMessage(message) = message {
            match message {
                UuidGeneratorMessage::VersionChanged(version) if version < VERSIONS.len() => {
                    if self.version != version {
                        self.version = version;
                        self.output.clear();
                        self.status = "ok";
                        if version == 2 && !self.v6_initialized {
                            self.v6_timestamp = current_unix_millis();
                            self.v6_node = random_node_id();
                            self.v6_initialized = true;
                        } else if version == 3 && !self.v7_initialized {
                            self.v7_timestamp = current_unix_millis();
                            self.v7_initialized = true;
                        }
                    }
                }
                UuidGeneratorMessage::VersionChanged(_) => {}
                UuidGeneratorMessage::NamespaceChanged(namespace) if namespace < 5 => {
                    self.namespace = namespace;
                    self.status = "ok";
                }
                UuidGeneratorMessage::NamespaceChanged(_) => {}
                UuidGeneratorMessage::CustomNamespaceChanged(value) => {
                    self.custom_namespace = value;
                    self.status = "ok";
                }
                UuidGeneratorMessage::NameChanged(value) => {
                    self.name = value;
                    self.status = "ok";
                }
                UuidGeneratorMessage::V6TimestampChanged(value) => {
                    self.v6_timestamp = value;
                    self.status = "ok";
                }
                UuidGeneratorMessage::V6NodeChanged(value) => {
                    self.v6_node = value;
                    self.status = "ok";
                }
                UuidGeneratorMessage::SetV6CurrentTime => {
                    self.v6_timestamp = current_unix_millis();
                    self.status = "ok";
                }
                UuidGeneratorMessage::RandomizeV6Node => {
                    self.v6_node = random_node_id();
                    self.status = "ok";
                }
                UuidGeneratorMessage::V7TimestampChanged(value) => {
                    self.v7_timestamp = value;
                    self.status = "ok";
                }
                UuidGeneratorMessage::SetV7CurrentTimeAndGenerate => {
                    self.v7_timestamp = current_unix_millis();
                    self.generate();
                }
                UuidGeneratorMessage::V8DataChanged(value) => {
                    self.v8_data = value;
                    self.status = "ok";
                }
                UuidGeneratorMessage::Generate => self.generate(),
                UuidGeneratorMessage::CopyOutput => {
                    return clipboard::write(self.output.clone());
                }
                UuidGeneratorMessage::SelectAllOutput => {
                    return text_input::select_all(Id::new(OUTPUT_TEXT_ID));
                }
                UuidGeneratorMessage::ReadOnlyOutput => {
                    return text_input::move_cursor_to_end(Id::new(OUTPUT_TEXT_ID));
                }
            }
        }
        Task::none()
    }
}

impl UuidGeneratorPage {
    const PAGE_NAME: &str = "uuid-generator";

    fn generate(&mut self) {
        match self.generate_uuid() {
            Ok(uuid) => {
                self.output = uuid.to_string();
                self.status = "ok";
            }
            Err(status) => {
                self.output.clear();
                self.status = status;
            }
        }
    }

    fn generate_uuid(&self) -> Result<Uuid, &'static str> {
        match self.version {
            0 => Ok(Uuid::new_v4()),
            1 => {
                let namespace = match self.namespace {
                    0 => Uuid::NAMESPACE_DNS,
                    1 => Uuid::NAMESPACE_OID,
                    2 => Uuid::NAMESPACE_URL,
                    3 => Uuid::NAMESPACE_X500,
                    4 => {
                        Uuid::parse_str(&self.custom_namespace).map_err(|_| "invalid-namespace")?
                    }
                    _ => return Err("invalid-namespace"),
                };
                Ok(Uuid::new_v5(&namespace, self.name.as_bytes()))
            }
            2 => {
                let millis = parse_timestamp(&self.v6_timestamp, MAX_V6_UNIX_MILLIS)?;
                let bytes = hex::decode(&self.v6_node).map_err(|_| "invalid-node-id")?;
                let node: [u8; 6] = bytes.try_into().map_err(|_| "invalid-node-id")?;
                let timestamp = Timestamp::from_unix(
                    &self.v6_context,
                    millis / 1000,
                    ((millis % 1000) * 1_000_000) as u32,
                );
                Ok(Uuid::new_v6(timestamp, &node))
            }
            3 => {
                let millis = parse_timestamp(&self.v7_timestamp, MAX_V7_UNIX_MILLIS)?;
                let timestamp = Timestamp::from_unix(
                    NoContext,
                    millis / 1000,
                    ((millis % 1000) * 1_000_000) as u32,
                );
                Ok(Uuid::new_v7(timestamp))
            }
            4 => {
                let bytes: [u8; 16] = self
                    .v8_data
                    .as_bytes()
                    .try_into()
                    .map_err(|_| "invalid-custom-data")?;
                Ok(Uuid::new_v8(bytes))
            }
            _ => unreachable!(),
        }
    }
}

fn parse_timestamp(input: &str, maximum: u64) -> Result<u64, &'static str> {
    input
        .parse::<u64>()
        .ok()
        .filter(|millis| *millis <= maximum)
        .ok_or("invalid-timestamp")
}

fn current_unix_millis() -> String {
    chrono::Utc::now().timestamp_millis().to_string()
}

fn random_node_id() -> String {
    let mut bytes = [0_u8; 6];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes[0] |= 1;
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::iced::core::widget::{Tree, tree::State};

    fn assert_tree_shape(actual: &Tree, expected: &Tree) {
        assert_eq!(actual.tag, expected.tag);
        assert_eq!(
            matches!(actual.state, State::None),
            matches!(expected.state, State::None)
        );
        assert_eq!(actual.children.len(), expected.children.len());
        for (actual_child, expected_child) in actual.children.iter().zip(&expected.children) {
            assert_tree_shape(actual_child, expected_child);
        }
    }

    #[test]
    fn switching_versions_preserves_widget_tree_state() {
        let mut page = UuidGeneratorPage::default();
        let mut tree = Tree::new(page.get_utility_page().as_widget());

        for version in [1, 2, 3, 4, 0, 1] {
            let _ = page.handle_message(Message::UuidGeneratorMessage(
                UuidGeneratorMessage::VersionChanged(version),
            ));
            let mut view = page.get_utility_page();
            tree.diff(view.as_widget_mut());
            assert_tree_shape(&tree, &Tree::new(view.as_widget()));
        }

        for namespace in [4, 0, 4] {
            let _ = page.handle_message(Message::UuidGeneratorMessage(
                UuidGeneratorMessage::NamespaceChanged(namespace),
            ));
            let mut view = page.get_utility_page();
            tree.diff(view.as_widget_mut());
            assert_tree_shape(&tree, &Tree::new(view.as_widget()));
        }
    }

    #[test]
    fn generates_version_five_for_standard_and_custom_namespaces() {
        let mut page = UuidGeneratorPage::default();
        page.version = 1;
        page.name = "example.org".to_string();
        let standard = page.generate_uuid().unwrap();
        assert_eq!(standard, Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"example.org"));

        page.namespace = 4;
        page.custom_namespace = Uuid::NAMESPACE_DNS.to_string();
        assert_eq!(page.generate_uuid().unwrap(), standard);
        page.custom_namespace = "invalid".to_string();
        assert_eq!(page.generate_uuid(), Err("invalid-namespace"));
    }

    #[test]
    fn generates_version_six_with_selected_time_and_node() {
        let mut page = UuidGeneratorPage::default();
        page.version = 2;
        page.v6_timestamp = "1645557742000".to_string();
        page.v6_node = "010203040506".to_string();
        let uuid = page.generate_uuid().unwrap();
        assert_eq!(uuid.get_version_num(), 6);
        assert_eq!(uuid.get_timestamp().unwrap().to_unix(), (1645557742, 0));
        assert_eq!(&uuid.as_bytes()[10..], &[1, 2, 3, 4, 5, 6]);

        page.v6_node = "123".to_string();
        assert_eq!(page.generate_uuid(), Err("invalid-node-id"));
        page.v6_node = "010203040506".to_string();
        page.v6_timestamp = (MAX_V6_UNIX_MILLIS + 1).to_string();
        assert_eq!(page.generate_uuid(), Err("invalid-timestamp"));
    }

    #[test]
    fn generates_version_seven_with_selected_milliseconds() {
        let mut page = UuidGeneratorPage::default();
        page.version = 3;
        page.v7_timestamp = "1645557742123".to_string();
        let uuid = page.generate_uuid().unwrap();
        assert_eq!(uuid.get_version_num(), 7);
        assert_eq!(
            uuid.get_timestamp().unwrap().to_unix(),
            (1645557742, 123_000_000)
        );

        page.v7_timestamp = "-1".to_string();
        assert_eq!(page.generate_uuid(), Err("invalid-timestamp"));
        page.v7_timestamp = (MAX_V7_UNIX_MILLIS + 1).to_string();
        assert_eq!(page.generate_uuid(), Err("invalid-timestamp"));
    }

    #[test]
    fn generates_version_eight_from_exactly_sixteen_utf_eight_bytes() {
        let mut page = UuidGeneratorPage::default();
        page.version = 4;
        page.v8_data = "abcdefghijklmnop".to_string();
        assert_eq!(
            page.generate_uuid().unwrap(),
            Uuid::new_v8(*b"abcdefghijklmnop")
        );

        page.v8_data = "abcdefghijklmnopq".to_string();
        assert_eq!(page.generate_uuid(), Err("invalid-custom-data"));
        page.v8_data = "é".repeat(8);
        assert!(page.generate_uuid().is_ok());
        page.v8_data.push('é');
        assert_eq!(page.generate_uuid(), Err("invalid-custom-data"));
    }

    #[test]
    fn now_button_updates_the_v7_input_and_output_together() {
        let mut page = UuidGeneratorPage::default();
        page.version = 3;
        page.v7_timestamp = "0".to_string();

        let before = chrono::Utc::now().timestamp_millis();
        let _ = page.handle_message(Message::UuidGeneratorMessage(
            UuidGeneratorMessage::SetV7CurrentTimeAndGenerate,
        ));
        let after = chrono::Utc::now().timestamp_millis();

        let millis = page.v7_timestamp.parse::<i64>().unwrap();
        assert!((before..=after).contains(&millis));
        let uuid = Uuid::parse_str(&page.output).unwrap();
        let (seconds, nanos) = uuid.get_timestamp().unwrap().to_unix();
        assert_eq!(seconds * 1000 + u64::from(nanos / 1_000_000), millis as u64);
    }
}
