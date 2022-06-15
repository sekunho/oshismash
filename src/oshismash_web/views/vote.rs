use maud::{html, Markup};

use crate::{
    oshismash::{
        vote::{Action, Stat},
        vtubers::{Stack, VTuber},
    },
    oshismash_web::components::icon,
};

pub fn render(stack: Stack) -> Markup {
    let current_vtuber = stack.get_current();

    html! {
        div class="flex-1 flex flex-col justify-center items-center" {
            // Cards
            div class="flex-1 relative w-full sm:w-2/3 lg:w-1/3" {
                @if let Some(vtuber) = current_vtuber {
                    @if let Some(_) = vtuber.next {
                        div id="card" class="top-10 left-0 right-0 mx-auto absolute rounded-lg shadow-lg opacity-50 bg-su-bg-2 dark:bg-su-dark-bg-2 w-10/12 h-full mx-auto" {
                        }
                    }

                    div id="card" class="top-5 left-0 right-0 mx-auto absolute rounded-lg shadow-lg opacity-70 bg-su-bg-2 dark:bg-su-dark-bg-2 w-11/12 h-full mx-auto" {
                    }
                    (card(vtuber))
                } @else {
                    (last_card())
                }
            }

            div class="flex mt-16 space-x-2.5" {
                // noscript {
                    (prev_vtuber(&stack))
                    (next_vtuber(&stack))
                // }

                @match stack.clone() {
                    Stack::NoPrev { current, vote_for_current, .. } => {
                        (pass(&current, &vote_for_current))
                        (smash(&current, &vote_for_current))
                    }

                    Stack::HasBoth { current, vote_for_current, .. } => {
                        (pass(&current, &vote_for_current))
                        (smash(&current, &vote_for_current))
                    }

                    Stack::NoCurrent { .. } => ("")
                }
            }

            @if let Some(stat) = stack.get_last_voted_stat() {
                span class="hidden md:block dark:text-su-dark-fg-1 mt-6" { (format!("What others voted for {}", stat.name)) }
                div class="flex justify-center space-x-2 w-full sm:w-2/3 md:w-1/3 mt-4 dark:text-su-dark-fg-1" {
                    div class="flex flex-col w-full items-end" {
                        span class="text-sm md:text-base font-bold mb-1 md:mb-2.5 text-right" { ("Passes") }
                        span
                            class="text-sm md:text-base rounded-md h-2 md:h-6 bg-gradient-to-l from-red-500 to-pink-500"
                            style=(style_percentage(stat.passes, stat.smashes + stat.passes)) {

                            }

                        span class="font-bold text-lg text-right" { (stat.passes) }
                    }

                    figure class="flex-none w-12 md:w-24 aspect-square bg-su-dark-bg-2 rounded-md" {
                        img class="object-cover object-top h-full w-full" src=(stat.img.as_ref().unwrap());
                    }

                    div class="flex flex-col w-full" {
                        span class="text-sm md:text-base font-bold mb-1 md:mb-2.5" { ("Smashes") }
                        span
                            class="text-sm md:text-base rounded-md h-2 md:h-6 bg-gradient-to-r from-cyan-500 to-blue-500"
                            style=(style_percentage(stat.smashes, stat.smashes + stat.passes)) {

                            }

                        span class="font-bold text-lg" { (stat.smashes) }
                    }
                }
            }
        }
    }
}

fn style_percentage(nominator: i64, denominator: i64) -> String {
    format!(
        "width: {}%;",
        (nominator as f64 / denominator as f64) * 100 as f64
    )
}

fn last_card() -> Markup {
    html! {
        div id="card" class="flex items-center justify-center absolute rounded-lg shadow-lg bg-su-bg-2 dark:bg-su-dark-bg-2 w-full h-full mx-auto" {
            span class="font-medium text-white text-2xl text-center" {
                ("You can touch grass now.")
            }
        }
    }
}

fn card(vtuber: &VTuber) -> Markup {
    html! {
        div id="card" class="absolute rounded-lg shadow-lg bg-su-bg-2 dark:bg-su-dark-bg-2 w-full h-full mx-auto" {
            figure class="h-full w-full rounded-lg relative" {
                img class="object-top object-cover h-full w-full rounded-lg" src=(vtuber.img);

                figcaption class="w-full left-0 bottom-0 rounded-b-lg absolute bg-gradient-to-t from-black p-4" {
                    div class="flex items-center space-x-2.5" {
                        h1 class="font-bold text-white text-3xl" {
                            (vtuber.name)
                        }

                        span class="text-white text-lg" {
                            (vtuber.org_name)
                        }
                    }

                    p class="text-lg text-white overflow-y-auto max-h-24" {
                        (vtuber.description)
                    }
                }


                div class="top-2 left-2 absolute space-y-2" {
                    (next_button(vtuber.next))
                    (prev_button(vtuber.prev))
                }
            }
        }
    }
}

// TODO: Use newtype
fn prev_button(vtuber_id: Option<i64>) -> Markup {
    html! {
        @match vtuber_id {
            Some(vtuber_id) =>  {
                form method="POST" action="/" {
                    input class="hidden" type="text" name="action" value="prev";
                    input class="hidden" type="text" name="vtuber_id" value=(vtuber_id);
                    button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1" {
                        p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                            (icon::chevron_down())
                        }
                    }
                }

            }

            None => {
                button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1 opacity-70 cursor-not-allowed" disabled {
                    p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                        (icon::chevron_down())
                    }
                }
            }
        }
    }
}

fn next_button(vtuber_id: Option<i64>) -> Markup {
    html! {
        @match vtuber_id {
            Some(vtuber_id) =>  {
                form method="POST" action="/" {
                    input class="hidden" type="text" name="action" value="next";
                    input class="hidden" type="text" name="vtuber_id" value=(vtuber_id);
                    button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1" {
                        p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                            (icon::chevron_up())
                        }
                    }
                }

            }

            None => {
                button class="rounded-full h-6 w-6 dark:bg-su-dark-bg-1 opacity-70 cursor-not-allowed" disabled {
                    p class="mx-auto h-5 w-5 dark:text-su-dark-fg-1 flex items-center justify-center" {
                        (icon::chevron_up())
                    }
                }
            }
        }
    }
}

fn smash(current_vtuber: &VTuber, current_vote: &Option<Action>) -> Markup {
    // let voted = vote_list.into_raw_parts
    html! {
        form method="POST" action="/" {
            input class="hidden" type="text" name="action" value="smashed";
            input class="hidden" type="text" name="vtuber_id" value=(current_vtuber.id);

            @match current_vote {
                Some(Action::Smashed) => {
                    button class="shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 bg-gradient-to-t from-cyan-500 to-blue-500" {
                        p class="mx-auto h-6 w-6 md:h-8 md:w-8 text-white flex items-center justify-center" { (icon::heart()) }
                    }
                }

                _ => {
                    button class="shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 border border-cyan-500 hover:border-0 hover:bg-gradient-to-t hover:from-cyan-500 hover:to-blue-500 text-cyan-500 hover:text-white" title="Smash" {
                        p class="mx-auto h-6 w-6 md:h-8 md:w-8 flex items-center justify-center" {
                            (icon::heart())
                        }
                    }
                },
            }
        }
    }
}

fn pass(current_vtuber: &VTuber, current_vote: &Option<Action>) -> Markup {
    html! {
        form method="POST" action="/" {
            input class="hidden" type="text" name="action" value="passed";
            input class="hidden" type="text" name="vtuber_id" value=(current_vtuber.id);

            @match current_vote {
                Some(Action::Passed) => {
                    button class="shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 bg-gradient-to-t from-red-500 to-pink-500" {
                        p class="mx-auto h-6 w-6 md:h-8 md:w-8 text-white flex items-center justify-center" { (icon::x()) }
                    }

                }

                _ => {
                    button class="shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 border border-red-500 hover:border-0 hover:bg-gradient-to-t hover:from-red-500 hover:to-pink-500 text-red-500 hover:text-white" title="Pass" {
                        p class="mx-auto h-6 w-6 md:h-8 md:w-8 flex items-center justify-center" {
                            (icon::x())
                        }
                    }
                },
            }
        }
    }
}

fn next_vtuber(stack: &Stack) -> Markup {
    let next_button = |vtuber_id: i64| {
        html! {
            a href=(format!("/{}", vtuber_id)) class="flex items-center justify-center shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 bg-su-bg-2 dark:bg-su-dark-bg-2" {
                p class="h-6 w-6 md:h-8 md:w-8 text-white flex items-center justify-center" {
                    (icon::chevron_up())
                }
            }
        }
    };

    let next_disabled_button = html! {
       button
           disabled
           class="cursor-not-allowed shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 bg-su-bg-2 dark:bg-su-dark-bg-2 opacity-50" {
            p class="mx-auto h-6 w-6 md:h-8 md:w-8 text-white flex items-center justify-center" {
                (icon::chevron_up())
            }
        }
    };

    match stack.get_current() {
        Some(current) => match current.next {
            Some(id) => next_button(id),
            None => next_disabled_button,
        },
        None => next_disabled_button,
    }
}

fn prev_vtuber(stack: &Stack) -> Markup {
    let prev_button = |vtuber_id: i64| {
        html! {
            a href=(format!("/{}", vtuber_id)) class="flex items-center justify-center shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 bg-su-bg-2 dark:bg-su-dark-bg-2" {
                p class="h-6 w-6 md:h-8 md:w-8 text-white flex items-center justify-center" {
                    (icon::chevron_down())
                }
            }
        }
    };

    let prev_disabled_button = html! {
       button
           disabled
           class="cursor-not-allowed shadow-md rounded-full h-12 w-12 md:h-14 md:w-14 bg-su-bg-2 dark:bg-su-dark-bg-2 opacity-50" {
            p class="mx-auto h-6 w-6 md:h-8 md:w-8 text-white flex items-center justify-center" {
                (icon::chevron_down())
            }
        }
    };

    match stack.get_current() {
        Some(current) => match current.prev {
            Some(id) => prev_button(id),
            None => prev_disabled_button,
        },
        None => match stack.get_last_voted_stat() {
            Some(Stat { vtuber_id, .. }) => prev_button(*vtuber_id),
            None => prev_disabled_button,
        },
    }
}
