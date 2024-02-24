//! Based loosely on https://www.npmjs.com/package/drag-drop-touch
use fluvio_wasm_timer::Instant;
use gloo_events::EventListenerOptions;
use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_hooks::use_event;

static DOUBLE_CLICK_MS: u128 = 500;
static CONTEXT_MENU_ACTION_MS: u32 = 900;
static MOVEMENT_EPSILON: i32 = 5;

fn should_handle(e: &TouchEvent) -> bool {
	!e.default_prevented() && e.touches().length() < 2
}

fn dispatch_mouse_event(e: &TouchEvent, name: &str, target: Option<web_sys::EventTarget>) -> bool {
	let Some(target) = target else {
		return false;
	};
	let mut event_args = web_sys::MouseEventInit::new();
	event_args
		.bubbles(true)
		.cancelable(true)
		.button(0)
		.buttons(1)
		.alt_key(e.alt_key())
		.ctrl_key(e.ctrl_key())
		.meta_key(e.meta_key())
		.shift_key(e.shift_key())
		.view(e.view().as_ref());
	if let Some(touch) = e.touches().get(0) {
		event_args
			.client_x(touch.client_x())
			.client_y(touch.client_y())
			.screen_x(touch.screen_x())
			.screen_y(touch.screen_y());
	}
	let Ok(event) = web_sys::MouseEvent::new_with_mouse_event_init_dict(name, &event_args) else {
		return false;
	};
	let Ok(_) = target.dispatch_event(&event) else {
		return false;
	};
	event.default_prevented()
}

fn find_draggable(target: Option<web_sys::EventTarget>) -> Option<web_sys::Element> {
	let Some(target) = target else {
		return None;
	};
	let mut element = target.dyn_ref::<web_sys::Element>().cloned();
	while let Some(subject) = element {
		if let Some(value_str) = subject.get_attribute("draggable") {
			if let Ok(value) = value_str.parse::<bool>() {
				if value {
					return Some(subject);
				}
			}
		}
		element = subject.parent_element();
	}
	None
}

fn determine_point(event: &TouchEvent) -> (i32, i32) {
	if let Some(touch) = event.touches().get(0) {
		return (touch.client_x(), touch.client_y());
	}
	return (event.page_x(), event.page_y());
}

fn get_touch_target(event: &TouchEvent) -> Option<web_sys::Element> {
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();
	let (x, y) = determine_point(event);
	// find the element at the pointer location and lowest in the
	// parental chain that accepts pointerEvents, according to css.
	let mut element = document.element_from_point(x as f32, y as f32);
	while let Some(subject) = element {
		if let Ok(Some(style)) = window.get_computed_style(&subject) {
			// if pointerEvents != none, then it accepts pointerEvents
			if let Ok(value) = style.get_property_value("pointerEvents") {
				if value != "none" {
					return Some(subject);
				}
			}
		}
		element = subject.parent_element();
	}
	None
}

fn get_movement_delta(start: (i32, i32), end: (i32, i32)) -> i32 {
	(end.0 - start.0).abs() + (end.1 - start.1).abs()
}

#[hook]
pub fn use_event_custom<T, F, E>(node: NodeRef, event_type: T, options: Option<EventListenerOptions>, callback: F)
where
	T: Into<std::borrow::Cow<'static, str>>,
	F: Fn(E) + 'static,
	E: From<wasm_bindgen::JsValue>,
{
	let callback = yew_hooks::use_latest(callback);

	use_effect_with((node, event_type.into()), move |(node, event_type)| {
		let window = gloo_utils::window();
		let node = node.get();
		// If we cannot get the wrapped `Node`, then we use `Window` as the default target of the event.
		let target = node.as_deref().map_or(&*window, |t| t);

		let options = match options {
			Some(options) => options,
			None => {
				// We should only set passive event listeners for `touchstart` and `touchmove`.
				// See here: https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#Improving_scrolling_performance_with_passive_listeners
				if event_type == "touchstart" || event_type == "touchmove" || event_type == "scroll" {
					EventListenerOptions::default()
				} else {
					EventListenerOptions::enable_prevent_default()
				}
			}
		};

		let listener =
			gloo_events::EventListener::new_with_options(target, event_type.clone(), options, move |event| {
				(*callback.current())(wasm_bindgen::JsValue::from(event).into());
			});

		move || drop(listener)
	});
}

#[hook]
pub fn use_touch_event_delegation(node: NodeRef) {
	let last_click = use_state(|| None::<Instant>);
	let drag_source = use_state(|| None::<web_sys::Element>);
	let down_evt_point = use_state(|| None::<(i32, i32)>);
	let last_touch = use_state(|| None::<web_sys::TouchEvent>);
	let last_target = use_state(|| None::<web_sys::EventTarget>);
	let started_dragging = use_state(|| false);
	let can_drop_here = use_state(|| false);

	let reset = Callback::from({
		let last_click = last_click.clone();
		let drag_source = drag_source.clone();
		let down_evt_point = down_evt_point.clone();
		let last_touch = last_touch.clone();
		let last_target = last_target.clone();
		let started_dragging = started_dragging.clone();
		let can_drop_here = can_drop_here.clone();
		move |_: ()| {
			last_click.set(None);
			drag_source.set(None);
			down_evt_point.set(None);
			last_touch.set(None);
			last_target.set(None);
			started_dragging.set(false);
			can_drop_here.set(false);
		}
	});

	use_event_custom(
		node.clone(),
		"touchstart",
		Some(EventListenerOptions::enable_prevent_default()),
		{
			let last_click = last_click.clone();
			let reset = reset.clone();
			let drag_source = drag_source.clone();
			let down_evt_point = down_evt_point.clone();
			let last_touch = last_touch.clone();
			let started_dragging = started_dragging.clone();
			move |e: TouchEvent| {
				log::debug!(target: "touch", "received touchstart");

				if !should_handle(&e) {
					return;
				}

				log::debug!(target: "touch", "processing touchstart");

				if let Some(last_click) = *last_click {
					if last_click.elapsed().as_millis() < DOUBLE_CLICK_MS {
						log::debug!(target: "touch", "doubleclick");
						if dispatch_mouse_event(&e, "dblclick", e.target()) {
							if e.cancelable() {
								e.prevent_default();
							}
							reset.emit(());
							return;
						}
					}
				}

				reset.emit(());

				let Some(draggable_element) = find_draggable(e.target()) else {
					return;
				};
				if dispatch_mouse_event(&e, "mousemove", e.target()) {
					return;
				}
				if dispatch_mouse_event(&e, "mousedown", e.target()) {
					return;
				}

				log::debug!(target: "touch", "starting drag on touchstart");

				drag_source.set(Some(draggable_element.clone()));
				down_evt_point.set(Some(determine_point(&e)));
				last_touch.set(Some(e.clone()));
				if e.cancelable() {
					e.prevent_default();
				}

				// show context menu if the user hasn't started dragging after a while
				Timeout::new(CONTEXT_MENU_ACTION_MS, {
					let drag_source = drag_source.clone();
					let instigating_event = e.clone();
					let subject = draggable_element.clone();
					let started_dragging = started_dragging.clone();
					let reset = reset.clone();
					move || {
						if *drag_source == Some(subject.clone()) && !*started_dragging {
							log::debug!(target: "touch", "initiating contextmenu");
							let target = subject.dyn_ref::<web_sys::EventTarget>().cloned();
							if dispatch_mouse_event(&instigating_event, "contextmenu", target) {
								reset.emit(());
							}
						}
					}
				})
				.forget();
			}
		},
	);
	use_event_custom(
		node.clone(),
		"touchmove",
		Some(EventListenerOptions::enable_prevent_default()),
		{
			let last_touch = last_touch.clone();
			let down_evt_point = down_evt_point.clone();
			let started_dragging = started_dragging.clone();
			let last_target = last_target.clone();
			let drag_source = drag_source.clone();
			let can_drop_here = can_drop_here.clone();
			move |e: TouchEvent| {
				log::debug!(target: "touch", "received touchmove");
				if !should_handle(&e) {
					return;
				}
				let Some(element_currently_over) = get_touch_target(&e) else {
					return;
				};
				let target_currently_over = element_currently_over.dyn_ref::<web_sys::EventTarget>().cloned();

				if dispatch_mouse_event(&e, "mousemove", target_currently_over.clone()) {
					log::debug!(target: "touch", "mousemove processed");
					last_touch.set(Some(e.clone()));
					if e.cancelable() {
						e.prevent_default();
					}
					return;
				}

				if let (Some(element_being_dragged), Some(start_point)) = (&*drag_source, *down_evt_point) {
					let current_point = determine_point(&e);
					let delta = get_movement_delta(start_point, current_point);
					if !*started_dragging && delta > MOVEMENT_EPSILON {
						log::debug!(target: "touch", "initiating dragging");
						let being_dragged_target = element_being_dragged.dyn_ref::<web_sys::EventTarget>().cloned();
						dispatch_mouse_event(&e, "dragstart", being_dragged_target);
						started_dragging.set(true);
						dispatch_mouse_event(&e, "dragenter", target_currently_over.clone());
					} else {
						log::debug!(target: "touch", "shouldnt initiate dragging, started? = {:?} delta={}", *started_dragging, delta);
					}
				} else {
					log::debug!(target: "touch", "invalid drag subject");
				}

				if !*started_dragging {
					log::debug!(target: "touch", "dragging not initiated");
					return;
				}

				log::debug!(target: "touch", "updating drag status");

				last_touch.set(Some(e.clone()));
				if e.cancelable() {
					log::debug!(target: "touch", "preventing scrolling");
					e.prevent_default(); // prevent scrolling
				}
				if target_currently_over != *last_target {
					dispatch_mouse_event(&e, "dragleave", (*last_target).clone());
					dispatch_mouse_event(&e, "dragenter", target_currently_over.clone());
					last_target.set(target_currently_over.clone());
				}
				can_drop_here.set(dispatch_mouse_event(&e, "dragover", target_currently_over.clone()));
			}
		},
	);
	let touchend = Callback::from({
		let last_touch = last_touch.clone();
		let started_dragging = started_dragging.clone();
		let drag_source = drag_source.clone();
		let can_drop_here = can_drop_here.clone();
		let reset = reset.clone();
		move |e: TouchEvent| {
			log::debug!(target: "touch", "received touchend/cancel");
			if !should_handle(&e) {
				return;
			}

			if let Some(last_touch) = &*last_touch {
				if dispatch_mouse_event(last_touch, "mouseup", e.target()) {
					if e.cancelable() {
						e.prevent_default();
					}
					return;
				}
			}

			if !*started_dragging {
				drag_source.set(None);
				if let Some(last_touch) = &*last_touch {
					dispatch_mouse_event(last_touch, "click", e.target());
				}
				last_click.set(Some(Instant::now()));
			}

			if let Some(drag_source) = &*drag_source {
				log::debug!(target: "touch", "dropping dragged");
				if let Some(last_touch) = &*last_touch {
					if !e.type_().contains("cancel") && *can_drop_here {
						dispatch_mouse_event(last_touch, "drop", (*last_target).clone());
					}
					let being_dragged_target = drag_source.dyn_ref::<web_sys::EventTarget>().cloned();
					dispatch_mouse_event(last_touch, "dragend", being_dragged_target);
				}
				reset.emit(());
			}
		}
	});
	use_event_custom(
		node.clone(),
		"touchend",
		Some(EventListenerOptions::enable_prevent_default()),
		{
			let touchend = touchend.clone();
			move |e: TouchEvent| {
				touchend.emit(e);
			}
		},
	);
	use_event_custom(
		node.clone(),
		"touchcancel",
		Some(EventListenerOptions::enable_prevent_default()),
		{
			let touchend = touchend.clone();
			move |e: TouchEvent| {
				touchend.emit(e);
			}
		},
	);
}
