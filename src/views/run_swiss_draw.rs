use crate::Route;
use dioxus::prelude::*;

const BLOG_CSS: Asset = asset!("/assets/styling/blog.css");


#[component]
pub fn Enter_Scores(sd_id: i64) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: BLOG_CSS }

        div {
            id: "Score Draw",

            // Content
            h1 { "This is Swiss Draw #{sd_id}!" }


        }
    }
}
