use crate::{
    async_callback,
    components::{PobColoredText, PobGems, PobTreePreview, PobTreeTable},
    memo,
    pob::{self, Element},
};
use ::pob::{PathOfBuilding, PathOfBuildingExt, SerdePathOfBuilding};
use shared::model::PasteId;
use std::rc::Rc;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

pub struct ViewPasteProps {
    pub id: PasteId,
    pub title: Option<String>,
    pub content: Rc<str>,
    pub pob: Rc<SerdePathOfBuilding>,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
enum CopyState {
    Ready,
    Progress,
    Done,
    Failed,
}

impl CopyState {
    fn name(&self) -> &'static str {
        match self {
            Self::Ready => "Copy",
            Self::Progress => "Copy",
            Self::Done => "Copied",
            Self::Failed => "Failed",
        }
    }
}

#[component(ViewPaste<G>)]
pub fn view_paste(
    ViewPasteProps {
        id,
        title,
        content,
        pob,
    }: ViewPasteProps,
) -> View<G> {
    let title = title.unwrap_or_else(|| pob::title(&*pob));

    let version = pob.max_tree_version().unwrap_or_default();

    let open_in_pob_url = id.to_pob_open_url();

    let notes = pob.notes().to_owned();
    let notes = if !notes.is_empty() {
        view! {
            div(class="flex-auto") {
                h3(class="text-lg dark:text-slate-100 text-slate-900 mb-2 mt-24 border-b border-solid") { "Notes" }
                pre(class="text-xs break-words whitespace-pre-line font-mono sm:ml-3") {
                    PobColoredText(notes)
                }
            }
        }
    } else {
        View::empty()
    };
    let tree_preview = if has_displayable_tree(&pob) {
        view! {
            div(class="basis-full") {
                h3(class="text-lg dark:text-slate-100 text-slate-900 mb-2 mt-24 border-b border-solid") { "Tree Preview" }
                PobTreePreview(Rc::clone(&pob))
            }
        }
    } else {
        View::empty()
    };

    let select_all = |event: web_sys::Event| {
        let s: HtmlTextAreaElement = event.target().unwrap().unchecked_into();
        let _ = s.focus();
        s.select();
    };

    let content_ref = NodeRef::new();
    let copy_state = Signal::new(CopyState::Ready);

    // TODO: figure out Signal clones and scopes
    let copy_to_clipboard = async_callback!(
        copy_state,
        content_ref,
        {
            use crate::utils::{document, from_ref};

            copy_state.set(CopyState::Progress);

            from_ref::<_, web_sys::HtmlTextAreaElement>(&content_ref).select();

            let document: web_sys::HtmlDocument = document();
            let state = if document.exec_command("copy").is_ok() {
                CopyState::Done
            } else {
                CopyState::Failed
            };

            let _ = document
                .get_selection()
                .unwrap()
                .unwrap()
                .remove_all_ranges();

            copy_state.set(state);
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            copy_state.set(CopyState::Ready);
        },
        *copy_state.get() == CopyState::Ready
    );

    let btn_copy_name = memo!(copy_state, copy_state.get().name());
    let btn_copy_disabled = memo!(copy_state, *copy_state.get() != CopyState::Ready);

    let core_stats = pob::summary::core_stats(&pob);
    let defense = pob::summary::defense(&pob);
    let offense = pob::summary::offense(&pob);
    let config = pob::summary::config(&pob);

    let summary = vec![core_stats, defense, offense, config]
        .into_iter()
        .map(render)
        .map(|stat| view! { div(class="flex-row gap-x-5") { (stat) } })
        .collect();
    let summary = View::new_fragment(summary);

    let src = crate::assets::ascendancy_image(pob.ascendancy_or_class_name()).unwrap_or_default();

    view! {
        div(class="flex flex-col md:flex-row gap-y-5 md:gap-x-3 mb-24") {
            div(class="flex-auto flex flex-col gap-y-2 -mt-[3px]") {
                h1(class="flex items-center text-xl mb-1 dark:text-slate-100 text-slate-900") {
                    img(src=src,
                        class="asc-image rounded-full mr-3 -ml-2",
                        onerror="this.style.display='none'") {}
                    span(class="pt-[3px]", data-marker-title="") { (title) }
                    sup(class="ml-1") { (version) }
                }
                (summary)
            }
            div(class="flex flex-col flex-initial gap-y-3 md:w-96") {
                textarea(
                    ref=content_ref,
                    on:click=select_all,
                    class="flex-auto resize-none text-sm break-all outline-none max-h-40 min-h-[5rem] dark:bg-slate-600 bg-slate-200 dark:text-slate-300 text-slate-700 rounded-sm shadow-sm pl-1",
                    data-marker-content="",
                    readonly=true
                ) {
                    (content)
                }
                div(class="text-right") {
                    button(
                        on:click=copy_to_clipboard,
                        disabled=*btn_copy_disabled.get(),
                        title="Copy to Clipboard",
                        class="hover:underline hover:cursor-pointer px-6 py-2 text-sm disabled:cursor-not-allowed inline-flex"
                    ) { (btn_copy_name.get()) }
                    a(
                        href=open_in_pob_url,
                        title="Open build in Path of Building, requires up to date PoB",
                        class="bg-sky-500 hover:bg-sky-700 hover:cursor-pointer px-6 py-2 text-sm rounded-lg font-semibold text-white disabled:opacity-50 disabled:cursor-not-allowed inline-flex"
                    ) { "Open" }
                }
            }
        }
        div(class="flex flex-wrap gap-x-10 gap-y-16") {
            div(class="flex-auto w-full lg:w-auto") {
                h3(class="text-lg dark:text-slate-100 text-slate-900 mb-2 border-b border-solid") { "Gems" }
                PobGems(pob.clone())
            }
            div(class="flex-1") {
                h3(class="text-lg dark:text-slate-100 text-slate-900 mb-2 border-b border-solid") { "Tree" }
                PobTreeTable(pob.clone())
            }
        }
        (tree_preview)
        (notes)
        div(class="h-[150px]") {}
    }
}

fn render<G: GenericNode>(elements: Vec<Element>) -> View<G> {
    View::new_fragment(
        elements
            .into_iter()
            .filter_map(|e| e.render_to_view())
            .collect(),
    )
}

fn has_displayable_tree(pob: &SerdePathOfBuilding) -> bool {
    let specs = pob.tree_specs();

    specs.len() > 1
        || specs
            .first()
            .map(|spec| spec.nodes.len() > 1)
            .unwrap_or(false)
}
