use serde::{Deserialize, Serialize};

const DC_CV: &str = "2.20211014.05.00";
const DC_CN: &str = "WEB";

// Request Body for Livechat
#[derive(Serialize, Deserialize, Debug)]
pub struct YoutubeChatRequestBody {
    context: YoutubeRequestBodyContext,
    continuation: String,
}
impl YoutubeChatRequestBody {
    pub fn new(continuation: String) -> Self {
        Self {
            context: YoutubeRequestBodyContext {
                client: YoutubeRequestBodyContextClient {
                    client_version: DC_CV.to_owned(),
                    client_name: DC_CN.to_owned(),
                },
            },
            continuation,
        }
    }
}

// Request Body for Send Message
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeMessageRequestBody {
    context: YoutubeRequestBodyContext,
    params: String,
    rich_message: YoutubeRichMessage,
}
impl YoutubeMessageRequestBody {
    pub fn new(params: String, message: String) -> Self {
        Self {
            rich_message: YoutubeRichMessage {
                text_segments: vec![YoutubeTextSegment { text: message }],
            },
            context: YoutubeRequestBodyContext {
                client: YoutubeRequestBodyContextClient {
                    client_version: DC_CV.to_owned(),
                    client_name: DC_CN.to_owned(),
                },
            },
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct YoutubeRichMessage {
    text_segments: Vec<YoutubeTextSegment>,
}

#[derive(Serialize, Deserialize, Debug)]
struct YoutubeTextSegment {
    text: String,
}

// General Request Body
#[derive(Serialize, Deserialize, Debug)]
pub struct YoutubeRequestBody {
    context: YoutubeRequestBodyContext,
    params: String,
}
impl YoutubeRequestBody {
    pub fn new(params: String) -> Self {
        Self {
            context: YoutubeRequestBodyContext {
                client: YoutubeRequestBodyContextClient {
                    client_version: DC_CV.to_owned(),
                    client_name: DC_CN.to_owned(),
                },
            },
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct YoutubeRequestBodyContext {
    client: YoutubeRequestBodyContextClient,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct YoutubeRequestBodyContextClient {
    client_version: String,
    client_name: String,
}

// Partial Body for LiveChat response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatResponse {
    pub continuation_contents: LiveChatContinuationContents,
}
impl LiveChatResponse {
    pub fn continuation_token(&self) -> String {
        match self
            .continuation_contents
            .live_chat_continuation
            .continuations
            .first()
        {
            Some(continuation) => continuation
                .invalidation_continuation_data
                .continuation
                .to_string(),
            None => String::from(""),
        }
    }

    pub fn timeout_ms(&self) -> u64 {
        match self
            .continuation_contents
            .live_chat_continuation
            .continuations
            .first()
        {
            Some(continuation) => continuation.invalidation_continuation_data.timeout_ms,
            None => 0,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatContinuationContents {
    pub live_chat_continuation: LiveChatContinuation,
}

#[derive(Deserialize, Debug)]
pub struct LiveChatContinuation {
    pub continuations: Vec<YoutubeContinuation>,
    pub actions: Option<Vec<ActionType>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeContinuation {
    pub invalidation_continuation_data: InvalidationContinuationData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InvalidationContinuationData {
    pub continuation: String,
    pub timeout_ms: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ActionType {
    #[serde(rename_all = "camelCase")]
    AddChatItem {
        add_chat_item_action: AddChatItemAction,
    },
    #[serde(rename_all = "camelCase")]
    MarkChatItemsByAuthorAsDeleted {
        mark_chat_items_by_author_as_deleted_action: MarkChatItemsByAuthorAsDeletedAction,
    },
    #[serde(rename_all = "camelCase")]
    AddLiveChatTickerItem {
        add_live_chat_ticker_item_action: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    MarkChatItemAsDeleted {
        mark_chat_item_as_deleted_action: MarkChatItemAsDeletedAction,
    },
    #[serde(rename_all = "camelCase")]
    AddBannerToLiveChatCommand {
        add_banner_to_live_chat_command: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    ReplaceChatItemAction {
        replace_chat_item_action: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    ShowLiveChatTooltipCommand {
      show_live_chat_tooltip_command: serde_json::Value,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddChatItemAction {
    pub item: ActionItem,
    pub client_id: Option<String>,
}
impl AddChatItemAction {
    pub fn get_message(&self) -> String {
        if let Some(text) = &self.item.live_chat_text_message_renderer {
            let mut output = String::from("");
            for run in text.message.runs.iter() {
                output = output.to_owned()
                    + match run {
                        MessageRun::MessageText { text } => text,
                        MessageRun::MessageEmoji { emoji, .. } => &emoji.emoji_id,
                    }
            }
            output
        } else {
            String::from("")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionItem {
    pub live_chat_text_message_renderer: Option<LiveChatTextMessageRenderer>,
    pub live_chat_paid_message_renderer: Option<LiveChatPaidMessageRenderer>,
    pub live_chat_membership_item_renderer: Option<LiveChatMembershipItemRenderer>,
    pub live_chat_paid_sticker_renderer: Option<LiveChatPaidStickerRenderer>,
    pub live_chat_viewer_engagement_message_renderer: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkChatItemsByAuthorAsDeletedAction {
    deleted_state_message: DeletedStateMessage,
    external_channel_id: String,
    show_original_content_message: ShowOriginalContentMessage,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeletedStateMessage {
    runs: Vec<MessageRun>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShowOriginalContentMessage {
    runs: Vec<MessageRun>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkChatItemAsDeletedAction {
    deleted_state_message: DeletedStateMessage,
    target_item_id: String,
}

/* MessageRun */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageRun {
    #[serde(rename_all = "camelCase")]
    MessageText { text: String },
    #[serde(rename_all = "camelCase")]
    MessageEmoji {
        emoji: Emoji,
        variant_ids: Option<Vec<String>>,
        is_custome_emoji: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    pub emoji_id: String,
    pub shortcuts: Option<Vec<String>>,
    pub search_terms: Option<Vec<String>>,
    pub supports_skin_tone: Option<bool>,
    pub image: Image,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub thumbnails: Vec<Thumbnail>,
    pub accessibility: Accessibility,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility {
    pub accessibility_data: AccessibilityData,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessibilityData {
    pub label: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<usize>,
    pub height: Option<usize>,
}
/* MessageRun End */

/* MessageRenderers */
/* MessageRenderersBase */
/* AuthorBadge */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorBadge {
    pub live_chat_author_badge_renderer: LiveChatAuthorBadgeRenderer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatAuthorBadgeRenderer {
    pub custom_thumbnail: Option<CustomThumbnail>,
    pub icon: Option<Icon>,
    pub tooltip: String,
    pub accessibility: Accessibility,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomThumbnail {
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    pub icon_type: String,
}
/* AuthorBadge End */

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageRendererBase {
    pub author_name: Option<AuthorName>,
    pub author_photo: AuthorPhoto,
    pub author_badges: Option<Vec<AuthorBadge>>,
    pub context_menu_endpoint: ContextMenuEndpoint,
    pub id: String,
    pub timestamp_usec: String,
    pub author_external_channel_id: String,
    pub context_menu_accessibility: Accessibility,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuEndpoint {
    pub click_tracking_params: Option<String>,
    pub command_metadata: CommandMetadata,
    pub live_chat_item_context_menu_endpoint: LiveChatItemContextMenuEndpoint,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiveChatItemContextMenuEndpoint {
    pub params: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata {
    pub web_command_metadata: WebCommandMetadata,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata {
    pub ignore_navigation: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorPhoto {
    pub thumbnails: Vec<Thumbnail>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorName {
    pub simple_text: String,
}
/* MessageRenderersBase End */

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatTextMessageRenderer {
    #[serde(flatten)]
    pub message_renderer_base: MessageRendererBase,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub runs: Vec<MessageRun>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatPaidMessageRenderer {
    #[serde(flatten)]
    pub message_renderer_base: MessageRendererBase,
    pub purchase_amount_text: PurchaseAmountText,
    pub header_background_color: isize,
    pub header_text_color: isize,
    pub body_background_color: isize,
    pub body_text_color: isize,
    pub author_name_text_color: isize,
    pub message: Option<Message>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiveChatPaidStickerRenderer {
    #[serde(flatten)]
    pub message_renderer_base: MessageRendererBase,
    #[serde(rename = "purchaseAmountText")]
    pub purchase_amount_text: PurchaseAmountText,
    pub sticker: Sticker,
    #[serde(rename = "moneyChipBackgroundColor")]
    pub money_chip_background_color: isize,
    #[serde(rename = "moneyChipTextColor")]
    pub money_chip_text_color: isize,
    #[serde(rename = "stickerDisplayWidth")]
    pub sticker_display_width: isize,
    #[serde(rename = "stickerDisplayHeight")]
    pub sticker_display_height: isize,
    #[serde(rename = "BackgroundColor")]
    pub background_color: isize,
    #[serde(rename = "authorNameTextColor")]
    pub author_name_text_color: isize,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sticker {
    pub thumbnails: Vec<Thumbnail>,
    pub accessibility: Accessibility,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PurchaseAmountText {
    #[serde(rename = "simpleText")]
    pub simple_text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiveChatMembershipItemRenderer {
    #[serde(flatten)]
    pub message_renderer_base: MessageRendererBase,
    pub header_subtext: HeaderSubtext,
    pub author_badges: Vec<serde_json::Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeaderSubtext {
    pub simple_text: String,
}
/*#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeaderSubText {
    pub runs: Vec<MessageRun>,
} */

// potentially a version that has SESSION_ID instead of DELEGATED_SESSION_ID
// Add SESSION_ID using enum functionality
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Credentials {
    pub sid: String,
    pub hsid: String,
    pub ssid: String,
    pub apisid: String,
    pub sapisid: String,
    pub delegated_session_id: String,
}