/*
 * niepce - crates/npc-fwk/src/toolkit/controller.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::rc::Rc;

/// Use this macro inside the impl to implement `imp()` and `imp_mut()`
///
/// ```rust,ignore
/// impl Controller for MyController {
///     npc_fwk::controller_imp_imp!(imp_)
/// }
/// ```
#[macro_export]
macro_rules! controller_imp_imp {
    ( $f:ident ) => {
        fn imp(
            &self,
        ) -> std::cell::Ref<'_, $crate::toolkit::ControllerImpl<Self::InMsg, Self::OutMsg>> {
            self.$f.borrow()
        }

        fn imp_mut(
            &self,
        ) -> std::cell::RefMut<'_, $crate::toolkit::ControllerImpl<Self::InMsg, Self::OutMsg>> {
            self.$f.borrow_mut()
        }
    };
}

// Type alias.
pub type ControllerImplCell<I, O> = std::cell::RefCell<ControllerImpl<I, O>>;

pub struct ControllerImpl<I, O> {
    tx: super::Sender<I>,
    rx: super::Receiver<I>,
    forwarder: Option<Box<dyn Fn(O)>>,
}

impl<I, O> Default for ControllerImpl<I, O> {
    fn default() -> ControllerImpl<I, O> {
        let (tx, rx) = super::channel();
        ControllerImpl::new(tx, rx)
    }
}

impl<I, O> ControllerImpl<I, O> {
    fn new(tx: super::Sender<I>, rx: super::Receiver<I>) -> Self {
        ControllerImpl {
            tx,
            rx,
            forwarder: None,
        }
    }
}

/// Controller allow encapsulating functionnality and receive message
/// to call it.
///
/// `InMsg` is the type of messages being sent (inbound) to it.
///
/// Implement [`Controller::dispatch`] to process the inbound
/// messages, and call [`Controller::start`] to set it up.  If the
/// controller is not supposed to receive inbound messages, `InMsg` to
/// `()`. Don't forget to call `controller_imp_imp!()` in your
/// implementation to generate some boilerplate.
///
/// ```rust,ignore
/// enum MyMsg {
///     Command1,
///     Command2(String),
/// }
///
/// #[derive(Default)]
/// struct MyController {
///     imp_: ControllerImplCell<Self::InMsg>,
/// }
///
/// impl Controller for MyController {
///     npc_fwk::controller_imp_imp!(imp_)
///
///     type InMsg = MyMsg;
///     type OutMsg = ();
///
///     fn dispatch(&self, msg: MyMsg) {
///         match msg {
///            MyMsg::Command1 => {}
///            MyMsg::Command2(s) => println!("{}", s),
///         }
///     }
/// }
/// ```
/// To send a message, call `Controller::send()`.
///
/// To get the sender to pass elsewhere call
/// `Controller::sender()`. The sender can be cloned.
///
///
/// ```rust,ignore
/// let ctrl = Rc::new(MyController::default());
/// <MyController as Controller>::start(&ctrl);
///
/// ctrl.send(MyMsg::Command1);
///
/// let sender = ctrl.sender();
/// sender.send(MyMsg::Command2("test".to_string())).await;
/// ```
pub trait Controller {
    type InMsg;
    type OutMsg;

    /// Start the controller event loop. This should be called
    /// creating the controller, if needed (i.e. need to process
    /// inbound messages). One pattern is to call it from the
    /// constructor that returns a `Rc<>`.
    ///
    /// This default implementation ought to be enough.
    fn start<T: Controller + 'static>(this: &Rc<T>) {
        let rx = this.imp().rx.clone();
        let ctrl = Rc::downgrade(this);
        super::channels::receiver_attach(
            rx,
            // We don't use glib::clone!() to be able to control the `Weak<>`
            // and report an error.
            move |e| {
                if let Some(ctrl) = ctrl.upgrade() {
                    ctrl.dispatch(e);
                } else {
                    err_out!("Weak controller is gone");
                }
            },
        );
    }

    /// Stop the dispatcher by closing the receiver.
    fn stop(&self) {
        self.imp().rx.close();
    }

    /// Set the forwarder the will pass [`Self::OutMsg`] to it.
    fn set_forwarder(&self, forwarder: Option<Box<dyn Fn(Self::OutMsg)>>) {
        self.imp_mut().forwarder = forwarder;
    }

    /// Get the receiver for the controller inbound messages.
    fn receiver(&self) -> super::Receiver<Self::InMsg> {
        self.imp().rx.clone()
    }

    /// Get the sender for the controller inbound messages.
    fn sender(&self) -> super::Sender<Self::InMsg> {
        self.imp().tx.clone()
    }

    /// Send an inbound message.
    ///
    /// XXX not sure about thread location. This is meant to be on the
    /// local context (main)
    fn send(&self, msg: Self::InMsg)
    where
        Self::InMsg: 'static,
    {
        super::send_async_local!(msg, self.imp().tx);
    }

    /// Send an outbound message.
    ///
    fn emit(&self, msg: Self::OutMsg) {
        if let Some(ref forwarder) = self.imp().forwarder {
            forwarder(msg);
        }
    }

    /// Notify the controller is ready.
    fn ready(&self) {
        dbg_out!("ready");
        self.on_ready();
    }

    /// What to do when ready.
    fn on_ready(&self) {}

    /// Dispatch input message, called by the event loop. See [`Controller::start`]
    fn dispatch(&self, _message: Self::InMsg) {}

    /// Return the implementation
    /// Implemented via controller_imp_imp!()
    fn imp(&self) -> std::cell::Ref<'_, ControllerImpl<Self::InMsg, Self::OutMsg>>;
    /// Return the mutable implementation
    /// Implemented via controller_imp_imp!()
    fn imp_mut(&self) -> std::cell::RefMut<'_, ControllerImpl<Self::InMsg, Self::OutMsg>>;
}
