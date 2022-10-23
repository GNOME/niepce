/*
 * niepce - crates/npc-fwk/src/toolkit/controller.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

use std::cell::{Ref, RefMut};
use std::rc::{Rc, Weak};

use uuid::Uuid;

pub struct ControllerImpl {
    id: Uuid,
    parent: Option<Weak<dyn Controller>>,
    children: Vec<Rc<dyn Controller>>,
}

impl Default for ControllerImpl {
    fn default() -> ControllerImpl {
        ControllerImpl {
            id: Uuid::new_v4(),
            parent: None,
            children: Vec::default(),
        }
    }
}

impl ControllerImpl {
    fn add(&mut self, child: &Rc<dyn Controller>) {
        self.children.push(child.clone());
    }

    fn remove(&mut self, child: &Rc<dyn Controller>) {
        for (i, c) in self.children.iter().enumerate() {
            if c.same(child) {
                self.children.remove(i);
                child.imp_mut().parent = None;
                break;
            }
        }
    }

    fn ready(&self) {
        self.children.iter().for_each(|child| child.ready());
    }
}

/// Convenience to create a controller.
pub fn new_controller<T: Controller + std::default::Default + 'static>() -> Rc<dyn Controller> {
    Rc::new(T::default())
}

pub trait Controller {
    fn id(&self) -> Uuid {
        self.imp().id
    }

    /// Two controller are the same if they have the same UUID.
    fn same(&self, other: &Rc<dyn Controller>) -> bool {
        self.imp().id == other.imp().id
    }

    /// Remove the controller
    fn add(&self, child: &Rc<dyn Controller>) {
        self.imp_mut().add(child);
        child.on_added();
    }

    /// Remove the controller
    fn remove(&self, child: &Rc<dyn Controller>) {
        self.imp_mut().remove(child)
    }

    fn parent(&self) -> Option<Weak<dyn Controller>> {
        self.imp().parent.as_ref().cloned()
    }

    fn set_parent(&self, parent: &Rc<dyn Controller>) {
        self.imp_mut().parent = Some(Rc::downgrade(parent));
    }

    /// Notify the controller has been added.
    fn on_added(&self) {}

    /// Notify the controller is ready. Will notify children and call on_ready()
    fn ready(&self) {
        self.imp().ready();
        self.on_ready();
    }

    /// What to do when ready.
    fn on_ready(&self);

    /// Return the implementation
    fn imp(&self) -> Ref<'_, ControllerImpl>;
    /// Return the mutable implementation
    fn imp_mut(&self) -> RefMut<'_, ControllerImpl>;
}

#[cfg(test)]
mod tests {
    use std::cell::{Ref, RefCell, RefMut};
    use std::rc::Rc;

    use uuid::Uuid;

    use super::{new_controller, Controller, ControllerImpl};

    #[derive(Default)]
    struct TestController {
        imp_: RefCell<ControllerImpl>,
    }

    impl Controller for TestController {
        /// Return the implementation
        fn imp(&self) -> Ref<'_, ControllerImpl> {
            self.imp_.borrow()
        }
        /// Return the mutable implementation
        fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
            self.imp_.borrow_mut()
        }

        fn on_ready(&self) {}
    }

    #[test]
    fn test_controller() {
        let testctrl1 = new_controller::<TestController>();

        assert_ne!(testctrl1.id(), Uuid::nil());

        let testctrl1_cloned = testctrl1.clone();
        assert!(testctrl1.same(&testctrl1_cloned));

        let testctrl2: Rc<dyn Controller> = Rc::new(TestController::default());
        assert_ne!(testctrl1.id(), testctrl2.id());
        assert!(!testctrl1.same(&testctrl2));
        assert!(!testctrl2.same(&testctrl1));

        let testctrl3: Rc<dyn Controller> = Rc::new(TestController::default());
        assert!(!testctrl1.same(&testctrl3));
        assert!(!testctrl2.same(&testctrl3));

        testctrl1.add(&testctrl2);
        assert_eq!(testctrl1.imp().children.len(), 1);
        testctrl1.add(&testctrl3);
        assert_eq!(testctrl1.imp().children.len(), 2);

        testctrl1.remove(&testctrl2);
        assert_eq!(testctrl1.imp().children.len(), 1);
        // try again
        testctrl1.remove(&testctrl2);
        assert_eq!(testctrl1.imp().children.len(), 1);

        testctrl1.remove(&testctrl3);
        assert!(testctrl1.imp().children.is_empty());
    }
}
