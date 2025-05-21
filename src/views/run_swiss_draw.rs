use crate::Route;
use dioxus::prelude::*;

const BLOG_CSS: Asset = asset!("/assets/styling/blog.css");

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
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
