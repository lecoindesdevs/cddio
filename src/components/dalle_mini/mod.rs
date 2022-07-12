use std::borrow::Cow;

use cddio_core::{ApplicationCommandEmbed, message};
use cddio_macros::component;
use serde::{Serialize, Deserialize};
use serenity::{client::Context, model::channel::AttachmentType};
use image::{ImageDecoder, ImageError, RgbaImage, GenericImage};

use crate::{log_warn, log_error};


pub struct DalleMini;


#[component]
impl DalleMini {
    #[command(name = "dalle_mini", description = "Dalle Mini generator")]
    async fn dalle_mini(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="What do you want to see ?")]
        what: String,
    ) {
        let delay_resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(delay_resp) => delay_resp,
            Err(e) => {
                log_error!("{}", e);
                return;
            }
        };
        let result = loop {
            let resp = match Self::fetch("https://bf.dallemini.ai/generate", what.clone()).await {
                Ok(resp) => resp,
                Err(e) => break Err(format!("{}", e))
            };
            let images = Self::parse(resp).await;
    
            let image = match Self::merge(images).await {
                Ok(image) => image,
                Err(e) => break Err(format!("{}", e))
            };
            let mut bytes: Vec<u8> = Vec::new();
            match image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageOutputFormat::Png) {
                Ok(_) => (),
                Err(e) => break Err(format!("{}", e))
            }
            let attacment = AttachmentType::Bytes { data: Cow::Borrowed(&bytes), filename: "dalle-mini.png".to_string() };
            match app_cmd.0.channel_id.send_message(ctx, |msg| {
                msg
                    .add_file(attacment)
                    .content(what)
            }).await {
                Ok(_) => (),
                Err(e) => break Err(format!("{}", e))
            };
            break Ok(());
        };
        let result = match result {
            Ok(_) => delay_resp.send_message(message::success("Image généré")).await,
            Err(e) => {
                log_error!("{}", e);
                delay_resp.send_message(message::Message::with_text(e)).await
            }
        };
        if let Err(e) = result {
            log_error!("{}", e);
        }
        
    }
}
#[derive(Serialize)]
struct DalleRequest {
    prompt: String,
}
#[derive(Deserialize)]
struct DalleResponse {
    images: Vec<String>
}

impl DalleMini {
    async fn fetch(url: &str, prompt: String) -> Result<DalleResponse, reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client.post(url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&DalleRequest{
                prompt
            })
            .send().await?;
        let body: DalleResponse = res.json().await?;
        
        Ok(body)
    }
    async fn parse(resp: DalleResponse) -> Vec<RgbaImage> {
        resp.images.into_iter().map(|b64img| {
            let b64img  =b64img.chars().into_iter().filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=').collect::<String>();
            let raw_data = match base64::decode(&b64img){
                Ok(data) => data,
                Err(e) => {
                    log_warn!("dalle_mini: Error decoding base64 image: {}", e);
                    return RgbaImage::new(0, 0);
                }
            };
            match image::load_from_memory(&raw_data) {
                Ok(img) => img.into_rgba8(),
                Err(e) => {
                    log_warn!("dalle_mini: unable to read image: {}", e);
                    RgbaImage::new(0, 0)
                }
            }
        }).collect()
    }
    async fn merge(images: Vec<RgbaImage>) -> image::ImageResult<image::RgbaImage> {
        assert_eq!(images.len(), 9);
        let small = (images[0].width(), images[0].height());
        const MARGIN:u32 = 10;
        let big = (small.0 * 3 + MARGIN * 2, small.1 * 3 + MARGIN * 2);

        let mut img = image::RgbaImage::new(
            big.0,
            big.1
        );
        for (i, img_i) in images.into_iter().enumerate() {
            let x = (i as u32 % 3) * (small.0 + MARGIN);
            let y = (i as u32 / 3) * (small.1 + MARGIN);
            img.copy_from(&img_i, x, y)?;
        }
        Ok(img)
    }
}