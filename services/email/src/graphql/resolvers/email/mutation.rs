use std::{env, time::SystemTime};

use async_graphql::{Context, Object, Result};
use lettre::{message::{Attachment, Body, MultiPart, SinglePart}, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use lib::utils::{custom_error::ExtendedError, models::Email};

#[derive(Default)]
pub struct EmailMutation;

#[Object]
impl EmailMutation {
    pub async fn send_email(&self, _ctx: &Context<'_>, email: Email) -> Result<String> {
        let smtp_user = env::var("SMTP_USER")
                        .expect("Missing the SMTP_USER environment variable.");
        let smtp_password = env::var("SMTP_PASSWORD")
                        .expect("Missing the SMTP_PASSWORD environment variable.");
        let smtp_server = env::var("SMTP_SERVER")
                        .expect("Missing the SMTP_SERVER environment variable.");

        let current_year = {
            let now = SystemTime::now();
            let datetime: chrono::DateTime<chrono::Utc> = now.into();
            datetime.format("%Y").to_string()
        };

        let email_title = email.title;
        let email_content = email.body;

        // let logo_image = fs::read("https://imagedelivery.net/fa3SWf5GIAHiTnHQyqU8IQ/5d0feb5f-2b15-4b86-9cf3-1f99372f4600/public")?;
        let logo_image = reqwest::get("https://imagedelivery.net/fa3SWf5GIAHiTnHQyqU8IQ/01f762dc-20a6-4842-30fb-2b2401c66200/public").await?.bytes().await?;

        println!("logo_image: {:?}", logo_image);

        let email_body = format!(r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <style>
                    /* General email body styling */
                    body {{
                        font-family: Arial, sans-serif;
                        margin: 0;
                        padding: 0;
                        background-color: #FFF7EF;
                    }}
                    .email-container {{
                        width: 100%;
                        max-width: 600px;
                        margin: 0 auto;
                        background-color: #ffffff;
                    }}
                    .header {{
                        background-color: #FFB161;
                        padding: 10px;
                        text-align: center;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                    }}
                    .header img {{
                        width: 200px;
                    }}
                    .content {{
                        padding: 20px;
                        color: #333333;
                    }}
                    .footer {{
                        background-color: #FFB161;
                        color: #ffffff;
                        text-align: center;
                        padding: 10px 0;
                    }}
                </style>
            </head>
            <body>
                <div class="email-container">
                    <!-- Header with logo -->
                    <div class="header">
                        <img src=cid:logo alt="Rusty Templates Logo">
                    </div>

                    <!-- Main content -->
                    <div class="content">
                        <!-- Replace the content below with your email-specific content -->
                        <h1>{email_title}</h1>
                        {email_content}
                        <!-- End of email-specific content -->
                    </div>
                        <!-- Footer -->
                        <div class="footer">
                            <div style="text-align: center; padding: 10px; font-size: 12px; color: #888888;">
                                <p>Rusty Templates | Tatu City, Kenya | info@rustytemplates.com</p>
                            </div>
                            &copy; {current_year} Rusty Templates. All rights reserved.
                        </div>
                    </div>
                </body>
                </html>
            "#
        );

        let logo_image_body = Body::new(logo_image.to_vec());

        let message = Message::builder()
            .from(format!("Rusty Templates <{}>", &smtp_user).parse()?)
            .reply_to(format!(" <{}>", &smtp_user).parse()?)
            .to(format!("{} <{}>", &email.recipient.full_name.unwrap_or(String::new()), &email.recipient.email_address).parse()?)
            .subject(&email.subject)
            .multipart(
                MultiPart::related()
                                            .singlepart(SinglePart::html(email_body))
                                            .singlepart(
                                                Attachment::new_inline(String::from("logo"))
                                                    .body(logo_image_body, "image/png".parse().unwrap()),
                                            ),
            )?;

        let creds = Credentials::new(smtp_user.to_owned(), smtp_password.to_owned());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay(&smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&message) {
            Ok(_) => Ok("Email sent successfully!".to_owned()),
            Err(e) => {
                println!("{:?}", e);
                Err(ExtendedError::new("Could not send email!", Some(400.to_string())).build())
            },
        }
    }
}
