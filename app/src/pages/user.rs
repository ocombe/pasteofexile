use crate::{future::LocalBoxFuture, model::PasteSummary, router::RoutedComponent, Meta, Result, utils::if_browser};
use sycamore::prelude::*;

pub struct Data {
    name: String,
    pastes: Vec<PasteSummary>,
}

impl<G: Html> RoutedComponent<G> for UserPage<G> {
    type RouteArg = String;

    fn from_context(name: Self::RouteArg, ctx: crate::Context) -> Result<Data> {
        Ok(Data {
            name,
            pastes: ctx.get_user().unwrap().to_vec(),
        })
    }

    fn from_hydration(name: Self::RouteArg, element: web_sys::Element) -> Result<Data> {
        let ssr = element
            .query_selector("[data-ssr]")
            .unwrap()
            .unwrap()
            .get_attribute("data-ssr")
            .unwrap_or_default();

        // TODO: maybe custom encoding instead of base64, just swap " and @ (a different character)
        let ssr = base64::decode_config(ssr, base64::URL_SAFE_NO_PAD).expect("b64 decode");
        let ssr = String::from_utf8(ssr).expect("utf8");

        let pastes = serde_json::from_str(&ssr).expect("deserialize");

        Ok(Data { name, pastes })
    }

    fn from_dynamic<'a>(name: Self::RouteArg) -> LocalBoxFuture<'a, Result<Data>> {
        Box::pin(async move {
            let pastes = crate::api::get_user(&name).await?;
            Ok(Data { name, pastes })
        })
    }

    fn meta(Data { name, .. }: &Data) -> Result<Meta> {
        let title = format!("Test {name}").into();
        let description = "description".into();
        let image = "".into();
        let color = "";

        Ok(Meta {
            title,
            description,
            image,
            color,
        })
    }
}

#[component(UserPage<G>)]
pub fn user_page(Data { pastes, .. }: Data) -> View<G> {
    let data_ssr = if_browser!(
        { String::new() }, 
        { base64::encode_config(serde_json::to_string(&pastes).unwrap(), base64::URL_SAFE_NO_PAD) }
    );

    let p = pastes
        .into_iter()
        .map(summary_to_view)
        .collect();
    let p = View::new_fragment(p);

    view! {
        div(data-ssr=data_ssr,
            class="flex flex-col gap-5") {
            (p)
        }
    }
}

fn summary_to_view<G: GenericNode>(summary: PasteSummary) -> View<G> {
    let url = summary.to_url();
    let asc = crate::assets::ascendancy_image(&summary.ascendancy).unwrap_or("");
    // TODO: properly implement for user in PoB and view_paste.rs component
    let open_in_pob_url = format!("pob://pobbin/{}:{}", summary.user.unwrap(), summary.id);

    view! {
        div(class="flex gap-3 p-3 border rounded-md last:border-0 border-slate-300") {
            img(src=asc,
                width=50, height=50,
                class="rounded-full h-min",
                onerror="this.style.visibility='hidden'") {}
            a(href=url, class="flex-auto") { (summary.title) }
            div() { 
                a(
                    href=open_in_pob_url,
                    title="Open build in Path of Building",
                    class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer px-6 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed inline-flex"
                 ) { "Open in PoB" }
            }
        }
    }
}
