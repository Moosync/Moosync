use crate::raw_parser::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use types::songs::Song;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistResponse {
    pub songs: Vec<Song>,
    pub next_page_token: Option<ContinuationToken>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuationToken {
    pub continuation: Option<String>,
    pub api_key: Option<String>,
    pub context: Context,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub response_context: Option<ResponseContext>,
    pub contents: Option<Contents>,
    // pub header: Option<Header2>,
    pub alerts: Option<Vec<Alert>>,
    // pub metadata: Option<Metadata>,
    // pub tracking_params: Option<String>,
    // pub topbar: Option<Topbar>,
    // pub microformat: Option<Microformat>,
    pub sidebar: Option<Value>,
    pub on_response_received_actions: Option<Vec<OnResponseReceivedAction>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnResponseReceivedAction {
    pub append_continuation_items_action: AppendContinuationItemsAction,
    pub click_tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendContinuationItemsAction {
    pub continuation_items: Vec<ContinuationItem>,
    pub target_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationItem {
    pub playlist_video_renderer: Option<PlaylistVideoRenderer>,
    pub continuation_item_renderer: Option<ContinuationItemRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseContext {
    pub service_tracking_params: Vec<ServiceTrackingParam>,
    pub main_app_web_response_context: MainAppWebResponseContext,
    pub web_response_context_extension_data: WebResponseContextExtensionData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceTrackingParam {
    pub service: String,
    pub params: Vec<Param>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MainAppWebResponseContext {
    pub logged_out: bool,
    pub tracking_param: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebResponseContextExtensionData {
    pub yt_config_data: Option<YtConfigData>,
    pub has_decorated: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YtConfigData {
    pub visitor_data: String,
    pub root_visual_element_type: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contents {
    pub two_column_browse_results_renderer: TwoColumnBrowseResultsRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoColumnBrowseResultsRenderer {
    pub tabs: Vec<Tab>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub tab_renderer: TabRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabRenderer {
    pub selected: bool,
    pub content: Option<Content>,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub section_list_renderer: SectionListRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionListRenderer {
    pub contents: Vec<Content2>,
    pub tracking_params: String,
    pub target_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content2 {
    pub item_section_renderer: Option<ItemSectionRenderer>,
    pub continuation_item_renderer: Option<ContinuationItemRenderer2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSectionRenderer {
    pub contents: Vec<Content3>,
    pub tracking_params: String,
    pub header: Option<Header>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content3 {
    pub playlist_video_list_renderer: PlaylistVideoListRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoListRenderer {
    pub contents: Vec<Content4>,
    pub playlist_id: String,
    pub is_editable: bool,
    pub can_reorder: bool,
    pub tracking_params: String,
    pub target_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content4 {
    pub playlist_video_renderer: Option<PlaylistVideoRenderer>,
    pub continuation_item_renderer: Option<ContinuationItemRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoRenderer {
    pub video_id: String,
    pub thumbnail: Thumbnail,
    pub title: Title,
    pub index: Index,
    pub short_byline_text: ShortBylineText,
    pub length_text: Option<LengthText>,
    pub navigation_endpoint: NavigationEndpoint2,
    pub length_seconds: Option<String>,
    pub tracking_params: String,
    pub is_playable: bool,
    pub menu: Menu,
    pub thumbnail_overlays: Vec<ThumbnailOverlay>,
    pub video_info: VideoInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub thumbnails: Vec<Thumbnail2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail2 {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub runs: Option<Vec<Run>>,
    pub accessibility: Accessibility,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility {
    pub accessibility_data: AccessibilityData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortBylineText {
    pub runs: Vec<Run2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run2 {
    pub text: String,
    pub navigation_endpoint: NavigationEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata,
    pub browse_endpoint: BrowseEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata {
    pub web_command_metadata: WebCommandMetadata,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint {
    pub browse_id: String,
    pub canonical_base_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LengthText {
    pub accessibility: Accessibility2,
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility2 {
    pub accessibility_data: AccessibilityData2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData2 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint2 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata2,
    pub watch_endpoint: WatchEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata2 {
    pub web_command_metadata: WebCommandMetadata2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata2 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint {
    pub video_id: String,
    pub playlist_id: String,
    pub index: i64,
    pub params: String,
    pub player_params: String,
    pub logging_context: LoggingContext,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext {
    pub vss_logging_context: VssLoggingContext,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig {
    pub common_config: CommonConfig,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Menu {
    pub menu_renderer: MenuRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuRenderer {
    pub items: Vec<Item>,
    pub tracking_params: String,
    pub accessibility: Accessibility3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub menu_service_item_renderer: MenuServiceItemRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuServiceItemRenderer {
    pub text: Text,
    pub icon: Icon,
    pub service_endpoint: ServiceEndpoint,
    pub tracking_params: String,
    pub has_separator: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text {
    pub runs: Option<Vec<Run3>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run3 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata3,
    pub signal_service_endpoint: Option<SignalServiceEndpoint>,
    pub share_entity_service_endpoint: Option<ShareEntityServiceEndpoint>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata3 {
    pub web_command_metadata: WebCommandMetadata3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata3 {
    pub send_post: bool,
    pub api_url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalServiceEndpoint {
    pub signal: String,
    pub actions: Vec<Action>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub click_tracking_params: String,
    pub add_to_playlist_command: AddToPlaylistCommand,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddToPlaylistCommand {
    pub open_miniplayer: bool,
    pub video_id: String,
    pub list_type: String,
    pub on_create_list_command: OnCreateListCommand,
    pub video_ids: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnCreateListCommand {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata4,
    pub create_playlist_service_endpoint: CreatePlaylistServiceEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata4 {
    pub web_command_metadata: WebCommandMetadata4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata4 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlaylistServiceEndpoint {
    pub video_ids: Vec<String>,
    pub params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareEntityServiceEndpoint {
    pub serialized_share_entity: String,
    pub commands: Vec<Command>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub click_tracking_params: String,
    pub open_popup_action: OpenPopupAction,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPopupAction {
    pub popup: Popup,
    pub popup_type: String,
    pub be_reused: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Popup {
    pub unified_share_panel_renderer: UnifiedSharePanelRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedSharePanelRenderer {
    pub tracking_params: String,
    pub show_loading_spinner: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility3 {
    pub accessibility_data: AccessibilityData3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData3 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlay {
    pub thumbnail_overlay_time_status_renderer: Option<ThumbnailOverlayTimeStatusRenderer>,
    pub thumbnail_overlay_now_playing_renderer: Option<ThumbnailOverlayNowPlayingRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlayTimeStatusRenderer {
    pub text: Text2,
    pub style: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text2 {
    pub accessibility: Accessibility4,
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility4 {
    pub accessibility_data: AccessibilityData4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData4 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlayNowPlayingRenderer {
    pub text: Text3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text3 {
    pub runs: Option<Vec<Run4>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run4 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub runs: Option<Vec<Run5>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run5 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationItemRenderer {
    pub trigger: String,
    pub continuation_endpoint: ContinuationEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata5,
    pub continuation_command: ContinuationCommand,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata5 {
    pub web_command_metadata: WebCommandMetadata5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata5 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationCommand {
    pub token: String,
    pub request: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub feed_filter_chip_bar_renderer: FeedFilterChipBarRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedFilterChipBarRenderer {
    pub contents: Vec<Content5>,
    pub tracking_params: String,
    pub style_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content5 {
    pub chip_cloud_chip_renderer: ChipCloudChipRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChipCloudChipRenderer {
    pub text: Text4,
    pub navigation_endpoint: NavigationEndpoint3,
    pub tracking_params: String,
    pub is_selected: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text4 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint3 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata6,
    pub browse_endpoint: BrowseEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata6 {
    pub web_command_metadata: WebCommandMetadata6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata6 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint2 {
    pub browse_id: String,
    pub nofollow: bool,
    pub navigation_type: String,
    pub params: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationItemRenderer2 {
    pub trigger: String,
    pub continuation_endpoint: ContinuationEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationEndpoint2 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata7,
    pub continuation_command: ContinuationCommand2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata7 {
    pub web_command_metadata: WebCommandMetadata7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata7 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationCommand2 {
    pub token: String,
    pub request: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header2 {
    pub playlist_header_renderer: PlaylistHeaderRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistHeaderRenderer {
    pub playlist_id: String,
    pub title: Title2,
    pub num_videos_text: NumVideosText,
    pub description_text: Option<DescriptionText>,
    pub owner_text: OwnerText,
    pub view_count_text: ViewCountText,
    pub share_data: ShareData,
    pub is_editable: bool,
    pub privacy: String,
    pub owner_endpoint: OwnerEndpoint,
    pub editable_details: EditableDetails,
    pub tracking_params: String,
    pub service_endpoints: Vec<ServiceEndpoint2>,
    pub stats: Vec<Stat>,
    pub brief_stats: Vec<BriefStat>,
    pub playlist_header_banner: PlaylistHeaderBanner,
    pub save_button: SaveButton,
    pub share_button: ShareButton,
    pub more_actions_menu: MoreActionsMenu,
    pub play_button: PlayButton,
    pub shuffle_play_button: ShufflePlayButton,
    pub on_description_tap: OnDescriptionTap,
    pub cinematic_container: CinematicContainer,
    pub byline: Vec<Byline>,
    pub description_tap_text: DescriptionTapText,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title2 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumVideosText {
    pub runs: Option<Vec<Run6>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run6 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionText {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerText {
    pub runs: Option<Vec<Run7>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run7 {
    pub text: String,
    pub navigation_endpoint: NavigationEndpoint4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint4 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata8,
    pub browse_endpoint: BrowseEndpoint3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata8 {
    pub web_command_metadata: WebCommandMetadata8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata8 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint3 {
    pub browse_id: String,
    pub canonical_base_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewCountText {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareData {
    pub can_share: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata9,
    pub browse_endpoint: BrowseEndpoint4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata9 {
    pub web_command_metadata: WebCommandMetadata9,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata9 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint4 {
    pub browse_id: String,
    pub canonical_base_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableDetails {
    pub can_delete: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpoint2 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata10,
    pub playlist_edit_endpoint: PlaylistEditEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata10 {
    pub web_command_metadata: WebCommandMetadata10,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata10 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistEditEndpoint {
    pub actions: Vec<Action2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action2 {
    pub action: String,
    pub source_playlist_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    pub runs: Option<Vec<Run8>>,
    pub simple_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run8 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BriefStat {
    pub runs: Option<Vec<Run9>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run9 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistHeaderBanner {
    pub hero_playlist_thumbnail_renderer: HeroPlaylistThumbnailRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroPlaylistThumbnailRenderer {
    pub thumbnail: Thumbnail3,
    pub max_ratio: f64,
    pub tracking_params: String,
    pub on_tap: OnTap,
    pub thumbnail_overlays: ThumbnailOverlays,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail3 {
    pub thumbnails: Vec<Thumbnail4>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail4 {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnTap {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata11,
    pub watch_endpoint: WatchEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata11 {
    pub web_command_metadata: WebCommandMetadata11,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata11 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint2 {
    pub video_id: String,
    pub playlist_id: String,
    pub player_params: String,
    pub logging_context: LoggingContext2,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext2 {
    pub vss_logging_context: VssLoggingContext2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext2 {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig2 {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig2 {
    pub common_config: CommonConfig2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig2 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlays {
    pub thumbnail_overlay_hover_text_renderer: ThumbnailOverlayHoverTextRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlayHoverTextRenderer {
    pub text: Text5,
    pub icon: Icon2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text5 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon2 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveButton {
    pub toggle_button_renderer: ToggleButtonRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleButtonRenderer {
    pub style: Style,
    pub size: Size,
    pub is_toggled: bool,
    pub is_disabled: bool,
    pub default_icon: DefaultIcon,
    pub toggled_icon: ToggledIcon,
    pub tracking_params: String,
    pub default_tooltip: String,
    pub toggled_tooltip: String,
    pub toggled_style: ToggledStyle,
    pub default_navigation_endpoint: DefaultNavigationEndpoint,
    pub accessibility_data: AccessibilityData5,
    pub toggled_accessibility_data: ToggledAccessibilityData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    pub style_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    pub size_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultIcon {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggledIcon {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggledStyle {
    pub style_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultNavigationEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata12,
    pub modal_endpoint: ModalEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata12 {
    pub web_command_metadata: WebCommandMetadata12,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata12 {
    pub ignore_navigation: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalEndpoint {
    pub modal: Modal,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modal {
    pub modal_with_title_and_button_renderer: ModalWithTitleAndButtonRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalWithTitleAndButtonRenderer {
    pub title: Title3,
    pub content: Option<Content6>,
    pub button: Button,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title3 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content6 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Button {
    pub button_renderer: ButtonRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text6,
    pub navigation_endpoint: NavigationEndpoint5,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text6 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint5 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata13,
    pub sign_in_endpoint: SignInEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata13 {
    pub web_command_metadata: WebCommandMetadata13,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata13 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInEndpoint {
    pub next_endpoint: NextEndpoint,
    pub idam_tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata14,
    pub browse_endpoint: BrowseEndpoint5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata14 {
    pub web_command_metadata: WebCommandMetadata14,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata14 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint5 {
    pub browse_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData5 {
    pub accessibility_data: AccessibilityData6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData6 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggledAccessibilityData {
    pub accessibility_data: AccessibilityData7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData7 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareButton {
    pub button_renderer: ButtonRenderer2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer2 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub service_endpoint: ServiceEndpoint3,
    pub icon: Icon3,
    pub tooltip: String,
    pub tracking_params: String,
    pub accessibility_data: AccessibilityData8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpoint3 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata15,
    pub share_entity_service_endpoint: ShareEntityServiceEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata15 {
    pub web_command_metadata: WebCommandMetadata15,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata15 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareEntityServiceEndpoint2 {
    pub serialized_share_entity: String,
    pub commands: Vec<Command2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command2 {
    pub click_tracking_params: String,
    pub open_popup_action: OpenPopupAction2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPopupAction2 {
    pub popup: Popup2,
    pub popup_type: String,
    pub be_reused: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Popup2 {
    pub unified_share_panel_renderer: UnifiedSharePanelRenderer2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedSharePanelRenderer2 {
    pub tracking_params: String,
    pub show_loading_spinner: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon3 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData8 {
    pub accessibility_data: AccessibilityData9,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData9 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoreActionsMenu {
    pub menu_renderer: MenuRenderer2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuRenderer2 {
    pub items: Vec<Item2>,
    pub tracking_params: String,
    pub accessibility: Accessibility5,
    pub target_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item2 {
    pub menu_navigation_item_renderer: MenuNavigationItemRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuNavigationItemRenderer {
    pub text: Text7,
    pub icon: Icon4,
    pub navigation_endpoint: NavigationEndpoint6,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text7 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon4 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint6 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata16,
    pub browse_endpoint: BrowseEndpoint6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata16 {
    pub web_command_metadata: WebCommandMetadata16,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata16 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint6 {
    pub browse_id: String,
    pub params: String,
    pub nofollow: bool,
    pub navigation_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility5 {
    pub accessibility_data: AccessibilityData10,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData10 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayButton {
    pub button_renderer: ButtonRenderer3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer3 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text8,
    pub icon: Icon5,
    pub navigation_endpoint: NavigationEndpoint7,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text8 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon5 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint7 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata17,
    pub watch_endpoint: WatchEndpoint3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata17 {
    pub web_command_metadata: WebCommandMetadata17,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata17 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint3 {
    pub video_id: String,
    pub playlist_id: String,
    pub player_params: String,
    pub logging_context: LoggingContext3,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext3 {
    pub vss_logging_context: VssLoggingContext3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext3 {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig3 {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig3 {
    pub common_config: CommonConfig3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig3 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShufflePlayButton {
    pub button_renderer: ButtonRenderer4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer4 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text9,
    pub icon: Icon6,
    pub navigation_endpoint: NavigationEndpoint8,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text9 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon6 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint8 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata18,
    pub watch_endpoint: WatchEndpoint4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata18 {
    pub web_command_metadata: WebCommandMetadata18,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata18 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint4 {
    pub video_id: String,
    pub playlist_id: String,
    pub params: String,
    pub player_params: String,
    pub logging_context: LoggingContext4,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext4 {
    pub vss_logging_context: VssLoggingContext4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext4 {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig4 {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig4 {
    pub common_config: CommonConfig4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig4 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnDescriptionTap {
    pub click_tracking_params: String,
    pub open_popup_action: OpenPopupAction3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPopupAction3 {
    pub popup: Popup3,
    pub popup_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Popup3 {
    pub fancy_dismissible_dialog_renderer: FancyDismissibleDialogRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FancyDismissibleDialogRenderer {
    pub dialog_message: DialogMessage,
    pub title: Title4,
    pub confirm_label: ConfirmLabel,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogMessage {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title4 {
    pub runs: Option<Vec<Run10>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run10 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmLabel {
    pub runs: Option<Vec<Run11>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run11 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CinematicContainer {
    pub cinematic_container_renderer: CinematicContainerRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CinematicContainerRenderer {
    pub background_image_config: BackgroundImageConfig,
    pub gradient_color_config: Vec<GradientColorConfig>,
    pub config: Config,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundImageConfig {
    pub thumbnail: Thumbnail5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail5 {
    pub thumbnails: Vec<Thumbnail6>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail6 {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientColorConfig {
    pub light_theme_color: i64,
    pub dark_theme_color: i64,
    pub start_location: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub light_theme_background_color: i64,
    pub dark_theme_background_color: i64,
    pub color_source_size_multiplier: i64,
    pub apply_client_image_blur: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Byline {
    pub playlist_byline_renderer: PlaylistBylineRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistBylineRenderer {
    pub text: Text10,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text10 {
    pub runs: Option<Option<Vec<Run12>>>,
    pub simple_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run12 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionTapText {
    pub runs: Option<Vec<Run13>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run13 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_with_button_renderer: AlertWithButtonRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertWithButtonRenderer {
    #[serde(rename = "type")]
    pub type_field: String,
    pub text: Text11,
    pub dismiss_button: DismissButton,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text11 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DismissButton {
    pub button_renderer: ButtonRenderer5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer5 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub icon: Icon7,
    pub tracking_params: String,
    pub accessibility_data: AccessibilityData11,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon7 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData11 {
    pub accessibility_data: AccessibilityData12,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData12 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub playlist_metadata_renderer: PlaylistMetadataRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistMetadataRenderer {
    pub title: String,
    pub android_appindexing_link: String,
    pub ios_appindexing_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topbar {
    pub desktop_topbar_renderer: DesktopTopbarRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTopbarRenderer {
    pub logo: Logo,
    pub searchbox: Searchbox,
    pub tracking_params: String,
    pub topbar_buttons: Vec<TopbarButton>,
    pub hotkey_dialog: HotkeyDialog,
    pub back_button: BackButton,
    pub forward_button: ForwardButton,
    pub a11y_skip_navigation_button: A11ySkipNavigationButton,
    pub voice_search_button: VoiceSearchButton,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logo {
    pub topbar_logo_renderer: TopbarLogoRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopbarLogoRenderer {
    pub icon_image: IconImage,
    pub tooltip_text: TooltipText,
    pub endpoint: Endpoint,
    pub tracking_params: String,
    pub override_entity_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconImage {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TooltipText {
    pub runs: Option<Vec<Run14>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run14 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata19,
    pub browse_endpoint: BrowseEndpoint7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata19 {
    pub web_command_metadata: WebCommandMetadata19,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata19 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint7 {
    pub browse_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Searchbox {
    pub fusion_searchbox_renderer: FusionSearchboxRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FusionSearchboxRenderer {
    pub icon: Icon8,
    pub placeholder_text: PlaceholderText,
    pub config: Config2,
    pub tracking_params: String,
    pub search_endpoint: SearchEndpoint,
    pub clear_button: ClearButton,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon8 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceholderText {
    pub runs: Option<Vec<Run15>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run15 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config2 {
    pub web_searchbox_config: WebSearchboxConfig,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSearchboxConfig {
    pub request_language: String,
    pub request_domain: String,
    pub has_onscreen_keyboard: bool,
    pub focus_searchbox: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEndpoint {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata20,
    pub search_endpoint: SearchEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata20 {
    pub web_command_metadata: WebCommandMetadata20,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata20 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEndpoint2 {
    pub query: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearButton {
    pub button_renderer: ButtonRenderer6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer6 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub icon: Icon9,
    pub tracking_params: String,
    pub accessibility_data: AccessibilityData13,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon9 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData13 {
    pub accessibility_data: AccessibilityData14,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData14 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopbarButton {
    pub topbar_menu_button_renderer: Option<TopbarMenuButtonRenderer>,
    pub button_renderer: Option<ButtonRenderer7>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopbarMenuButtonRenderer {
    pub icon: Icon10,
    pub menu_request: MenuRequest,
    pub tracking_params: String,
    pub accessibility: Accessibility6,
    pub tooltip: String,
    pub style: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon10 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuRequest {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata21,
    pub signal_service_endpoint: SignalServiceEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata21 {
    pub web_command_metadata: WebCommandMetadata21,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata21 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalServiceEndpoint2 {
    pub signal: String,
    pub actions: Vec<Action3>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action3 {
    pub click_tracking_params: String,
    pub open_popup_action: OpenPopupAction4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPopupAction4 {
    pub popup: Popup4,
    pub popup_type: String,
    pub be_reused: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Popup4 {
    pub multi_page_menu_renderer: MultiPageMenuRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiPageMenuRenderer {
    pub tracking_params: String,
    pub style: String,
    pub show_loading_spinner: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility6 {
    pub accessibility_data: AccessibilityData15,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData15 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer7 {
    pub style: String,
    pub size: String,
    pub text: Text12,
    pub icon: Icon11,
    pub navigation_endpoint: NavigationEndpoint9,
    pub tracking_params: String,
    pub target_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text12 {
    pub runs: Option<Vec<Run16>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run16 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon11 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint9 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata22,
    pub sign_in_endpoint: SignInEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata22 {
    pub web_command_metadata: WebCommandMetadata22,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata22 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInEndpoint2 {
    pub idam_tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyDialog {
    pub hotkey_dialog_renderer: HotkeyDialogRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyDialogRenderer {
    pub title: Title5,
    pub sections: Vec<Section>,
    pub dismiss_button: DismissButton2,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title5 {
    pub runs: Option<Vec<Run17>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run17 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub hotkey_dialog_section_renderer: HotkeyDialogSectionRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyDialogSectionRenderer {
    pub title: Title6,
    pub selection_option: Option<Vec<Options>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title6 {
    pub runs: Option<Vec<Run18>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run18 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub hotkey_dialog_section_option_renderer: HotkeyDialogSectionOptionRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyDialogSectionOptionRenderer {
    pub label: Label,
    pub hotkey: String,
    pub hotkey_accessibility_label: Option<HotkeyAccessibilityLabel>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub runs: Option<Vec<Run19>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run19 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyAccessibilityLabel {
    pub accessibility_data: AccessibilityData16,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData16 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DismissButton2 {
    pub button_renderer: ButtonRenderer8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer8 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text13,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text13 {
    pub runs: Option<Vec<Run20>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run20 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackButton {
    pub button_renderer: ButtonRenderer9,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer9 {
    pub tracking_params: String,
    pub command: Command3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command3 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata23,
    pub signal_service_endpoint: SignalServiceEndpoint3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata23 {
    pub web_command_metadata: WebCommandMetadata23,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata23 {
    pub send_post: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalServiceEndpoint3 {
    pub signal: String,
    pub actions: Vec<Action4>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action4 {
    pub click_tracking_params: String,
    pub signal_action: SignalAction,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalAction {
    pub signal: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardButton {
    pub button_renderer: ButtonRenderer10,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer10 {
    pub tracking_params: String,
    pub command: Command4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command4 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata24,
    pub signal_service_endpoint: SignalServiceEndpoint4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata24 {
    pub web_command_metadata: WebCommandMetadata24,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata24 {
    pub send_post: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalServiceEndpoint4 {
    pub signal: String,
    pub actions: Vec<Action5>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action5 {
    pub click_tracking_params: String,
    pub signal_action: SignalAction2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalAction2 {
    pub signal: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A11ySkipNavigationButton {
    pub button_renderer: ButtonRenderer11,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer11 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text14,
    pub tracking_params: String,
    pub command: Command5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text14 {
    pub runs: Option<Vec<Run21>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run21 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command5 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata25,
    pub signal_service_endpoint: SignalServiceEndpoint5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata25 {
    pub web_command_metadata: WebCommandMetadata25,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata25 {
    pub send_post: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalServiceEndpoint5 {
    pub signal: String,
    pub actions: Vec<Action6>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action6 {
    pub click_tracking_params: String,
    pub signal_action: SignalAction3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalAction3 {
    pub signal: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSearchButton {
    pub button_renderer: ButtonRenderer12,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer12 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub service_endpoint: ServiceEndpoint4,
    pub icon: Icon13,
    pub tooltip: String,
    pub tracking_params: String,
    pub accessibility_data: AccessibilityData19,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpoint4 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata26,
    pub signal_service_endpoint: SignalServiceEndpoint6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata26 {
    pub web_command_metadata: WebCommandMetadata26,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata26 {
    pub send_post: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalServiceEndpoint6 {
    pub signal: String,
    pub actions: Vec<Action7>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action7 {
    pub click_tracking_params: String,
    pub open_popup_action: OpenPopupAction5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPopupAction5 {
    pub popup: Popup5,
    pub popup_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Popup5 {
    pub voice_search_dialog_renderer: VoiceSearchDialogRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSearchDialogRenderer {
    pub placeholder_header: PlaceholderHeader,
    pub prompt_header: PromptHeader,
    pub example_query1: ExampleQuery1,
    pub example_query2: ExampleQuery2,
    pub prompt_microphone_label: PromptMicrophoneLabel,
    pub loading_header: LoadingHeader,
    pub connection_error_header: ConnectionErrorHeader,
    pub connection_error_microphone_label: ConnectionErrorMicrophoneLabel,
    pub permissions_header: PermissionsHeader,
    pub permissions_subtext: PermissionsSubtext,
    pub disabled_header: DisabledHeader,
    pub disabled_subtext: DisabledSubtext,
    pub microphone_button_aria_label: MicrophoneButtonAriaLabel,
    pub exit_button: ExitButton,
    pub tracking_params: String,
    pub microphone_off_prompt_header: MicrophoneOffPromptHeader,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceholderHeader {
    pub runs: Option<Vec<Run22>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run22 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptHeader {
    pub runs: Option<Vec<Run23>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run23 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleQuery1 {
    pub runs: Option<Vec<Run24>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run24 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleQuery2 {
    pub runs: Option<Vec<Run25>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run25 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptMicrophoneLabel {
    pub runs: Option<Vec<Run26>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run26 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadingHeader {
    pub runs: Vec<Run27>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run27 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionErrorHeader {
    pub runs: Vec<Run28>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run28 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionErrorMicrophoneLabel {
    pub runs: Vec<Run29>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run29 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsHeader {
    pub runs: Vec<Run30>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run30 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsSubtext {
    pub runs: Vec<Run31>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run31 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisabledHeader {
    pub runs: Vec<Run32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run32 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisabledSubtext {
    pub runs: Vec<Run33>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run33 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrophoneButtonAriaLabel {
    pub runs: Vec<Run34>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run34 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitButton {
    pub button_renderer: ButtonRenderer13,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer13 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub icon: Icon12,
    pub tracking_params: String,
    pub accessibility_data: AccessibilityData17,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon12 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData17 {
    pub accessibility_data: AccessibilityData18,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData18 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrophoneOffPromptHeader {
    pub runs: Vec<Run35>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run35 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon13 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData19 {
    pub accessibility_data: AccessibilityData20,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData20 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Microformat {
    pub microformat_data_renderer: MicroformatDataRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicroformatDataRenderer {
    pub url_canonical: String,
    pub title: String,
    pub description: String,
    pub thumbnail: Thumbnail7,
    pub site_name: String,
    pub app_name: String,
    pub android_package: String,
    pub ios_app_store_id: String,
    pub ios_app_arguments: String,
    pub og_type: String,
    pub url_applinks_web: String,
    pub url_applinks_ios: String,
    pub url_applinks_android: String,
    pub url_twitter_ios: String,
    pub url_twitter_android: String,
    pub twitter_card_type: String,
    pub twitter_site_handle: String,
    pub schema_dot_org_type: String,
    pub noindex: bool,
    pub unlisted: bool,
    pub link_alternates: Vec<LinkAlternate>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail7 {
    pub thumbnails: Vec<Thumbnail8>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail8 {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkAlternate {
    pub href_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sidebar {
    pub playlist_sidebar_renderer: PlaylistSidebarRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSidebarRenderer {
    pub items: Vec<Item3>,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item3 {
    // pub playlist_sidebar_primary_info_renderer: Option<PlaylistSidebarPrimaryInfoRenderer>,
    // pub playlist_sidebar_secondary_info_renderer: Option<PlaylistSidebarSecondaryInfoRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSidebarPrimaryInfoRenderer {
    pub thumbnail_renderer: ThumbnailRenderer,
    pub title: Title7,
    pub stats: Vec<Stat2>,
    pub menu: Menu2,
    pub thumbnail_overlays: Vec<ThumbnailOverlay2>,
    pub navigation_endpoint: NavigationEndpoint15,
    pub description: Description,
    pub show_more_text: ShowMoreText,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailRenderer {
    pub playlist_video_thumbnail_renderer: Option<PlaylistVideoThumbnailRenderer>,
    pub playlist_custom_thumbnail_renderer: Option<PlaylistVideoThumbnailRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoThumbnailRenderer {
    pub thumbnail: Thumbnail9,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail9 {
    pub thumbnails: Vec<Thumbnail10>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail10 {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title7 {
    pub runs: Vec<Run36>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run36 {
    pub text: String,
    pub navigation_endpoint: NavigationEndpoint10,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint10 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata27,
    pub watch_endpoint: WatchEndpoint5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata27 {
    pub web_command_metadata: WebCommandMetadata27,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata27 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint5 {
    pub video_id: String,
    pub playlist_id: String,
    pub player_params: String,
    pub logging_context: LoggingContext5,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext5 {
    pub vss_logging_context: VssLoggingContext5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext5 {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig5 {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig5 {
    pub common_config: CommonConfig5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig5 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat2 {
    pub runs: Option<Vec<Run37>>,
    pub simple_text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run37 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Menu2 {
    pub menu_renderer: MenuRenderer3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuRenderer3 {
    pub items: Vec<Item4>,
    pub tracking_params: String,
    pub top_level_buttons: Vec<TopLevelButton>,
    pub accessibility: Accessibility8,
    pub target_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item4 {
    pub menu_navigation_item_renderer: MenuNavigationItemRenderer2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuNavigationItemRenderer2 {
    pub text: Text15,
    pub icon: Icon14,
    pub navigation_endpoint: NavigationEndpoint11,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text15 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon14 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint11 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata28,
    pub browse_endpoint: Option<BrowseEndpoint8>,
    pub modal_endpoint: Option<ModalEndpoint2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata28 {
    pub web_command_metadata: WebCommandMetadata28,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata28 {
    pub url: Option<String>,
    pub web_page_type: Option<String>,
    pub root_ve: Option<i64>,
    pub api_url: Option<String>,
    pub ignore_navigation: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint8 {
    pub browse_id: String,
    pub params: String,
    pub nofollow: bool,
    pub navigation_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalEndpoint2 {
    pub modal: Modal2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modal2 {
    pub modal_with_title_and_button_renderer: ModalWithTitleAndButtonRenderer2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalWithTitleAndButtonRenderer2 {
    pub title: Title8,
    pub content: Option<Content7>,
    pub button: Button2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title8 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content7 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Button2 {
    pub button_renderer: ButtonRenderer14,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer14 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text16,
    pub navigation_endpoint: NavigationEndpoint12,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text16 {
    pub runs: Vec<Run38>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run38 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint12 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata29,
    pub sign_in_endpoint: SignInEndpoint3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata29 {
    pub web_command_metadata: WebCommandMetadata29,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata29 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInEndpoint3 {
    pub next_endpoint: NextEndpoint2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextEndpoint2 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata30,
    pub browse_endpoint: BrowseEndpoint9,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata30 {
    pub web_command_metadata: WebCommandMetadata30,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata30 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint9 {
    pub browse_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelButton {
    pub toggle_button_renderer: Option<ToggleButtonRenderer2>,
    pub button_renderer: Option<ButtonRenderer16>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleButtonRenderer2 {
    pub style: Style2,
    pub size: Size2,
    pub is_toggled: bool,
    pub is_disabled: bool,
    pub default_icon: DefaultIcon2,
    pub toggled_icon: ToggledIcon2,
    pub tracking_params: String,
    pub default_tooltip: String,
    pub toggled_tooltip: String,
    pub default_navigation_endpoint: DefaultNavigationEndpoint2,
    pub accessibility_data: AccessibilityData21,
    pub toggled_accessibility_data: ToggledAccessibilityData2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Style2 {
    pub style_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Size2 {
    pub size_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultIcon2 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggledIcon2 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultNavigationEndpoint2 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata31,
    pub modal_endpoint: ModalEndpoint3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata31 {
    pub web_command_metadata: WebCommandMetadata31,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata31 {
    pub ignore_navigation: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalEndpoint3 {
    pub modal: Modal3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modal3 {
    pub modal_with_title_and_button_renderer: ModalWithTitleAndButtonRenderer3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalWithTitleAndButtonRenderer3 {
    pub title: Title9,
    pub content: Option<Content8>,
    pub button: Button3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title9 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content8 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Button3 {
    pub button_renderer: ButtonRenderer15,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer15 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text17,
    pub navigation_endpoint: NavigationEndpoint13,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text17 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint13 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata32,
    pub sign_in_endpoint: SignInEndpoint4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata32 {
    pub web_command_metadata: WebCommandMetadata32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata32 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInEndpoint4 {
    pub next_endpoint: NextEndpoint3,
    pub idam_tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextEndpoint3 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata33,
    pub browse_endpoint: BrowseEndpoint10,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata33 {
    pub web_command_metadata: WebCommandMetadata33,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata33 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint10 {
    pub browse_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData21 {
    pub accessibility_data: AccessibilityData22,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData22 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggledAccessibilityData2 {
    pub accessibility_data: AccessibilityData23,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData23 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer16 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub service_endpoint: Option<ServiceEndpoint5>,
    pub icon: Icon15,
    pub accessibility: Accessibility7,
    pub tooltip: String,
    pub tracking_params: String,
    pub navigation_endpoint: Option<NavigationEndpoint14>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpoint5 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata34,
    pub share_entity_service_endpoint: ShareEntityServiceEndpoint3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata34 {
    pub web_command_metadata: WebCommandMetadata34,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata34 {
    pub send_post: bool,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareEntityServiceEndpoint3 {
    pub serialized_share_entity: String,
    pub commands: Vec<Command6>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command6 {
    pub click_tracking_params: String,
    pub open_popup_action: OpenPopupAction6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPopupAction6 {
    pub popup: Popup6,
    pub popup_type: String,
    pub be_reused: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Popup6 {
    pub unified_share_panel_renderer: UnifiedSharePanelRenderer3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedSharePanelRenderer3 {
    pub tracking_params: String,
    pub show_loading_spinner: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon15 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility7 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint14 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata35,
    pub watch_endpoint: WatchEndpoint6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata35 {
    pub web_command_metadata: WebCommandMetadata35,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata35 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint6 {
    pub video_id: String,
    pub playlist_id: String,
    pub params: String,
    pub player_params: String,
    pub logging_context: LoggingContext6,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext6 {
    pub vss_logging_context: VssLoggingContext6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext6 {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig6 {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig6 {
    pub common_config: CommonConfig6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig6 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accessibility8 {
    pub accessibility_data: AccessibilityData24,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityData24 {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlay2 {
    pub thumbnail_overlay_side_panel_renderer: ThumbnailOverlaySidePanelRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailOverlaySidePanelRenderer {
    pub text: Text18,
    pub icon: Icon16,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text18 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon16 {
    pub icon_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint15 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata36,
    pub watch_endpoint: WatchEndpoint7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata36 {
    pub web_command_metadata: WebCommandMetadata36,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata36 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint7 {
    pub video_id: String,
    pub playlist_id: String,
    pub player_params: String,
    pub logging_context: LoggingContext7,
    pub watch_endpoint_supported_onesie_config: WatchEndpointSupportedOnesieConfig7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingContext7 {
    pub vss_logging_context: VssLoggingContext7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VssLoggingContext7 {
    pub serialized_context_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpointSupportedOnesieConfig7 {
    #[serde(rename = "html5PlaybackOnesieConfig")]
    pub html5playback_onesie_config: Html5PlaybackOnesieConfig7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Html5PlaybackOnesieConfig7 {
    pub common_config: CommonConfig7,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig7 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowMoreText {
    pub runs: Vec<Run39>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run39 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSidebarSecondaryInfoRenderer {
    pub video_owner: VideoOwner,
    pub button: Button4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOwner {
    pub video_owner_renderer: VideoOwnerRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOwnerRenderer {
    pub thumbnail: Thumbnail11,
    pub title: Title10,
    pub navigation_endpoint: NavigationEndpoint17,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail11 {
    pub thumbnails: Vec<Thumbnail12>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail12 {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title10 {
    pub runs: Vec<Run40>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run40 {
    pub text: String,
    pub navigation_endpoint: NavigationEndpoint16,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint16 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata37,
    pub browse_endpoint: BrowseEndpoint11,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata37 {
    pub web_command_metadata: WebCommandMetadata37,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata37 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint11 {
    pub browse_id: String,
    pub canonical_base_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint17 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata38,
    pub browse_endpoint: BrowseEndpoint12,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata38 {
    pub web_command_metadata: WebCommandMetadata38,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata38 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint12 {
    pub browse_id: String,
    pub canonical_base_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Button4 {
    pub button_renderer: ButtonRenderer17,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer17 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text19,
    pub navigation_endpoint: NavigationEndpoint18,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text19 {
    pub runs: Vec<Run41>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run41 {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint18 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata39,
    pub modal_endpoint: ModalEndpoint4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata39 {
    pub web_command_metadata: WebCommandMetadata39,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata39 {
    pub ignore_navigation: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalEndpoint4 {
    pub modal: Modal4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modal4 {
    pub modal_with_title_and_button_renderer: ModalWithTitleAndButtonRenderer4,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalWithTitleAndButtonRenderer4 {
    pub title: Title11,
    pub content: Option<Content9>,
    pub button: Button5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title11 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content9 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Button5 {
    pub button_renderer: ButtonRenderer18,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRenderer18 {
    pub style: String,
    pub size: String,
    pub is_disabled: bool,
    pub text: Text20,
    pub navigation_endpoint: NavigationEndpoint19,
    pub tracking_params: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text20 {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint19 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata40,
    pub sign_in_endpoint: SignInEndpoint5,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata40 {
    pub web_command_metadata: WebCommandMetadata40,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata40 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInEndpoint5 {
    pub next_endpoint: NextEndpoint4,
    pub continue_action: String,
    pub idam_tag: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextEndpoint4 {
    pub click_tracking_params: String,
    pub command_metadata: CommandMetadata41,
    pub browse_endpoint: BrowseEndpoint13,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandMetadata41 {
    pub web_command_metadata: WebCommandMetadata41,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCommandMetadata41 {
    pub url: String,
    pub web_page_type: String,
    pub root_ve: i64,
    pub api_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseEndpoint13 {
    pub browse_id: String,
}
