use oauth2::{
    basic::{BasicErrorResponseType, BasicTokenType},
    Client, CsrfToken, EmptyExtraTokenFields, RevocationErrorResponseType, StandardErrorResponse,
    StandardRevocableToken, StandardTokenIntrospectionResponse, StandardTokenResponse,
};

pub type OAuthTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;
pub type OAuth2Client = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    OAuthTokenResponse,
    BasicTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;

pub type OAuth2Verifier = Option<(
    oauth2::Client<
        oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
        oauth2::basic::BasicTokenType,
        oauth2::StandardTokenIntrospectionResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        >,
        oauth2::StandardRevocableToken,
        oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    >,
    oauth2::PkceCodeVerifier,
    CsrfToken,
)>;
