use crate::domain::{Email, EmailClient};
use aws_config::SdkConfig;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;
use color_eyre::eyre::Result;
use secrecy::ExposeSecret;

pub struct SESEmailClient {
    sender: Email,
    ses_client: aws_sdk_sesv2::Client,
}

impl SESEmailClient {
    pub fn new(sender: Email, sdk_config: &SdkConfig) -> Self {
        let ses_client = Client::new(sdk_config);

        Self { sender, ses_client }
    }
}

#[async_trait::async_trait]
impl EmailClient for SESEmailClient {
    #[tracing::instrument(name = "Sending email", skip_all)]
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        let sender = self.sender.as_ref().expose_secret();
        let recipient = recipient.as_ref().expose_secret();
        let body_text = content;
        let body_html = format!("<html><body>{}</body></html>", content);

        // Create the email content
        let email_content = EmailContent::builder()
            .simple(
                Message::builder()
                    .subject(Content::builder().data(subject).build()?)
                    .body(
                        Body::builder()
                            .text(Content::builder().data(body_text).build()?)
                            .html(Content::builder().data(body_html).build()?)
                            .build(),
                    )
                    .build(),
            )
            .build();

        // Send the email
        let send_email_output = self
            .ses_client
            .send_email()
            .from_email_address(sender)
            .destination(Destination::builder().to_addresses(recipient).build())
            .content(email_content)
            .send()
            .await?;

        if let Some(id) = send_email_output.message_id() {
            tracing::debug!("Email was sent successfully with message id: {}", id);
        }

        Ok(())
    }
}

// @todo: add unit tests
