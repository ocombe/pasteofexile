use crate::{Meta, Prefetch};
use sycamore::prelude::*;

pub struct HeadArgs {
    pub meta: Meta,
    pub prefetch: Vec<Prefetch>,
    pub preload: Vec<Prefetch>,
}

#[component(Head<G>)]
pub fn head(args: HeadArgs) -> View<G> {
    let meta = args.meta;
    let title = meta.title.clone();
    let image = match meta.image.is_empty() {
        true => crate::assets::logo().into(),
        false => meta.image,
    };

    let preload = args
        .preload
        .into_iter()
        .map(|preload| {
            let typ = preload.typ();
            let href = preload.into_url();
            view! { link(rel="preload", href=href, as=typ) }
        })
        .collect::<Vec<_>>();
    let preload = View::new_fragment(preload);

    let prefetch = args
        .prefetch
        .into_iter()
        .map(|prefetch| {
            let href = prefetch.into_url();
            view! {
                link(rel="prefetch", href=href)
            }
        })
        .collect::<Vec<_>>();
    let prefetch = View::new_fragment(prefetch);

    let meta_title = meta.title.clone();
    let meta_description = meta.description.clone();
    view! {
        title { (title) }
        meta(name="title", content=meta_title)
        meta(name="description", content=meta_description)
        meta(property="og:type", content="website")
        meta(property="og:site_name", content="Paste of Exile - pobb.in")
        meta(property="og:title", content=meta.title)
        meta(property="og:description", content=meta.description)
        meta(property="og:image", content=image)
        meta(name="theme-color", content=meta.color)
        link(type="application/json+oembed", href=meta.oembed)
        (preload)
        (prefetch)
    }
}
