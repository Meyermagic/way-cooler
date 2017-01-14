//! Container types

use uuid::Uuid;

pub static MIN_SIZE: Size = Size { w: 80u32, h: 40u32 };

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::{Geometry, ResizeEdge, Point, Size, VIEW_FULLSCREEN};

use super::borders::{Borders, Color, Drawable, SimpleDraw};

/// A handle to either a view or output
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handle {
    View(WlcView),
    Output(WlcOutput)
}

/// Types of containers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    /// Root container, only one exists
    Root,
    /// WlcOutput/Monitor
    Output,
    /// A workspace
    Workspace,
    /// A Container, houses views and other containers
    Container,
    /// A view (window)
    View
}

impl ContainerType {
    /// Whether this container can be used as the parent of another
    pub fn can_have_child(self, other: ContainerType) -> bool {
        use self::ContainerType::*;
        match self {
            Root => other == Output,
            Output => other == Workspace,
            Workspace => other == Container,
            Container => other == Container || other == View,
            View => false
        }
    }
}

/// Layout mode for a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical
}

/// Represents an item in the container tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    /// Root node of the container
    Root(Uuid),
    /// Output
    Output {
        /// Handle to the wlc
        handle: WlcOutput,
        /// Optional background for the output
        background: Option<WlcView>,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// Workspace
    Workspace {
        /// Name of the workspace
        name: String,
        /// The size of the workspace on the screen.
        /// Might be different if there is e.g a bar present
        size: Size,
        /// `Vec` of all children that are fullscreen.
        /// This is used to disable certain features while there is a fullscreen
        /// (e.g: focus switching, resizing, and moving containers)
        fullscreen_c: Vec<Uuid>,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// Container
    Container {
        /// How the container is layed out
        layout: Layout,
        /// If the container is floating
        floating: bool,
        /// If the container is fullscreen
        fullscreen: bool,
        /// The geometry of the container, relative to the parent container
        geometry: Geometry,
        /// UUID associated with container, client program can use container
        id: Uuid,
        /// The border drawn to the screen
        borders: Option<Borders>,
    },
    /// View or window
    View {
        /// The wlc handle to the view
        handle: WlcView,
        /// Whether this view is floating
        floating: bool,
        /// Effective geometry. This is the size of the container including
        /// borders and gaps. It does _not_ change when an app becomes
        /// fullscreen. E.g to get the fullscreen size use `handle.get_geometry`
        effective_geometry: Geometry,
        /// UUID associated with container, client program can use container
        id: Uuid,
        /// The border drawn to the screen
        borders: Option<Borders>,
    }
}


impl Container {
    /// Creates a new root container.
    pub fn new_root() -> Container {
        Container::Root(Uuid::new_v4())
    }
    /// Creates a new output container with the given output
    pub fn new_output(handle: WlcOutput) -> Container {
        Container::Output {
            handle: handle,
            background: None,
            id: Uuid::new_v4()
        }
    }

    /// Creates a new workspace container with the given name and size.
    /// Usually the size is the same as the output it resides on,
    /// unless there is a bar or something.
    pub fn new_workspace(name: String, size: Size) -> Container {
        Container::Workspace {
            name: name,
            size: size,
            fullscreen_c: Vec::new(),
            id: Uuid::new_v4()
        }
    }

    /// Creates a new container
    pub fn new_container(geometry: Geometry) -> Container {
        Container::Container {
            layout: Layout::Horizontal,
            floating: false,
            fullscreen: false,
            geometry: geometry,
            id: Uuid::new_v4(),
            // TODO Remove hardcoded 0
            borders: Some(Borders::new(geometry, 0))
        }
    }

    /// Creates a new view container with the given handle
    pub fn new_view(handle: WlcView) -> Container {
        let geometry = handle.get_geometry()
            .expect("View had no geometry");
        let output_size = handle.get_output().get_resolution().unwrap();
        let output_geo = Geometry {
            origin: Point { x: 0, y: 0 },
            size: output_size
        };
        Container::View {
            handle: handle,
            floating: false,
            effective_geometry: geometry,
            id: Uuid::new_v4(),
            // TODO Remove hardcoded thickness
            borders: Some(Borders::new(output_geo, 50))
        }
    }

    /// Sets the visibility of this container
    pub fn set_visibility(&mut self, visibility: bool) {
        let mask = if visibility { 1 } else { 0 };
        if let Some(handle) = self.get_handle() {
            match handle {
                Handle::View(view) => {
                    view.set_mask(mask)
                },
                _ => {},
            }
        }
    }

    /// Gets the type of this container
    pub fn get_type(&self) -> ContainerType {
        match *self {
            Container::Root(_) => ContainerType::Root,
            Container::Output { .. } => ContainerType::Output,
            Container::Workspace { .. } => ContainerType::Workspace,
            Container::Container { .. } => ContainerType::Container,
            Container::View { .. } => ContainerType::View
        }
    }

    /// Gets the view handle of the view container, if this is a view container
    pub fn get_handle(&self) -> Option<Handle> {
        match *self {
            Container::View { ref handle, ..} => Some(Handle::View(handle.clone())),
            Container::Output { ref handle, .. } => Some(Handle::Output(handle.clone())),
            _ => None
        }
    }

    /// Gets the name of the workspace, if this container is a workspace.
    pub fn get_name(&self) -> Option<&str> {
        match *self {
            Container::Workspace { ref name, ..} => Some(name),
            _ => None
        }
    }

    /// Gets the geometry of the container, if the container has one.
    /// Root: Returns None
    /// Workspace/Output: Size is the size of the screen, origin is just 0,0
    /// Container/View: Size is the size of the container,
    /// origin is the coordinates relative to the parent container.
    pub fn get_geometry(&self) -> Option<Geometry> {
        match *self {
            Container::Root(_)  => None,
            Container::Output { ref handle, .. } => Some(Geometry {
                origin: Point { x: 0, y: 0 },
                size: handle.get_resolution()
                    .expect("Couldn't get output resolution")
            }),
            Container::Workspace { size, .. } => Some(Geometry {
                origin: Point { x: 0, y: 0},
                size: size
            }),
            Container::Container { geometry, .. } => Some(geometry),
            Container::View { ref handle, ..} =>
                handle.get_geometry(),
        }
    }

    /// Sets the geometry behind the container. Does nothing if container is root.
    ///
    /// For view you need to set the appropriate edges (which can be empty).
    /// If you are not intending to set the geometry of a view, simply pass `ResizeEdge::empty()`
    pub fn set_geometry(&mut self, edges: ResizeEdge, geo: Geometry) {
        match *self {
            Container::Root(_) => error!("Tried to set the geometry of the root!"),
            Container::Output { ref handle, .. } => {
                handle.set_resolution(geo.size, 1);
            },
            Container::Workspace { ref mut size, .. } => {
                *size = geo.size;
            },
            Container::Container { ref mut geometry, .. } => {
                *geometry = geo;
            },
            Container::View { ref handle, ref mut borders, .. } => {
                handle.set_geometry(edges, geo);
                borders.as_mut().map(|b| b.reallocate_buffer(geo));
            }
        }
    }

    pub fn set_layout(&mut self, new_layout: Layout) -> Result<(), String>{
        match *self {
            Container::Container { ref mut layout, .. } => *layout = new_layout,
            ref other => return Err(
                format!("Can only set the layout of a container, not {:?}",
                        other))
        }
        Ok(())
    }

    pub fn get_id(&self) -> Uuid {
        match *self {
            Container::Root(id) | Container::Output { id, .. } |
            Container::Workspace { id, .. } | Container::Container { id, .. } |
            Container::View { id, .. } => {
                id
            }
        }
    }

    pub fn floating(&self) -> bool {
        match *self {
            Container::View { floating, .. } | Container::Container { floating, .. } => floating,
            Container::Workspace { .. } | Container::Output { .. } | Container::Root(_) => false
        }
    }


    // TODO Make these set_* functions that can fail return a proper error type.

    /// If not set on a view or container, error is returned telling what
    /// container type that this function was (incorrectly) called on.
    pub fn set_floating(&mut self, val: bool) -> Result<ContainerType, ContainerType> {
        let c_type = self.get_type();
        match *self {
            Container::View { ref mut floating, .. } |
            Container::Container { ref mut floating, .. } => {
                *floating = val;
                Ok(c_type)
            },
            _ => {
                Err(c_type)
            }
        }
    }

    /// Sets the fullscreen flag on the container to the specified value.
    ///
    /// If called on a non View/Container, then returns an Err with the wrong type.
    pub fn set_fullscreen(&mut self, val: bool) -> Result<(), ContainerType> {
        let c_type = self.get_type();
        match *self {
            Container::View { handle, effective_geometry, .. } => {
                handle.set_state(VIEW_FULLSCREEN, val);
                if !val {
                    handle.set_geometry(ResizeEdge::empty(), effective_geometry);
                }
                Ok(())
            },
            Container::Container { ref mut fullscreen, .. } => {
                *fullscreen = val;
                Ok(())
            },
            _ => Err(c_type)
        }
    }

    /// Determines if a container is fullscreen.
    ///
    /// Workspaces, Outputs, and the Root are never fullscreen.
    pub fn fullscreen(&self) -> bool {
        match *self {
            Container::View { handle, .. } => {
                handle.get_state().intersects(VIEW_FULLSCREEN)
            },
            Container::Container { fullscreen, .. } => fullscreen,
            _ => false
        }
    }

    /// Updates the workspace (`self`) that the `id` resides in to reflect
    /// whether the container with the `id` is fullscreen (`toggle`).
    ///
    /// If called with a non-workspace an Err is returned with
    /// the incorrect type.
    pub fn update_fullscreen_c(&mut self, id: Uuid, toggle: bool)
                               -> Result<(), ContainerType> {
        let c_type = self.get_type();
        match *self {
            Container::Workspace { ref mut fullscreen_c, .. } => {
                if !toggle {
                    match fullscreen_c.iter().position(|c_id| *c_id == id) {
                        Some(index) => { fullscreen_c.remove(index); },
                        None => {}
                    }
                } else {
                    fullscreen_c.push(id);
                }
                Ok(())
            },
            _ => Err(c_type)
        }
    }

    /// If the container is a workspace, returns the children in the workspace that
    /// are fullscreen. The last child is the one visible to the user.
    ///
    /// Computes in O(1) time.
    ///
    /// If the container is not a workspace, None is returned.
    pub fn fullscreen_c(&self) -> Option<&Vec<Uuid>> {
        match *self {
            Container::Workspace { ref fullscreen_c, .. } =>
                Some(fullscreen_c),
            _ => None
        }
    }

    /// Gets the name of the container.
    ///
    /// Container::Root: returns simply the string "Root Container"
    /// Container::Output: The name of the output
    /// Container::Workspace: The name of the workspace
    /// Container::Container: Layout style (e.g horizontal)
    /// Container::View: The name taken from `WlcView`
    pub fn name(&self) -> String {
        match  *self {
            Container::Root(_)  => "Root Container".into(),
            Container::Output { handle, .. } => {
                handle.get_name()
            },
            Container::Workspace { ref name, .. } => name.clone(),
            Container::Container { layout, .. } => {
                format!("{:?}", layout)
            },
            Container::View { handle, ..} => {
                handle.get_title()
            }
        }
    }


    pub fn render_borders(&mut self) {
        match *self {
            Container::View { ref mut borders, .. } |
            Container::Container { ref mut borders, .. } => {
                if let Some(borders) = borders.as_mut() {
                    borders.render();
                }
            },
            _ => panic!("Tried to render a non-view / non-container")
        }
    }

    pub fn draw_borders(&mut self) {
        // TODO Eventually, we should use an enum to choose which way to draw the
        // border, but for now this will do.
        match *self {
            Container::View { ref mut borders, handle, .. } => {
                if let Some(borders_) = borders.take() {
                    let geometry = handle.get_geometry()
                        .expect("View had no geometry");
                    // TODO Don't hard code color
                    *borders = SimpleDraw::new(borders_.enable_cairo().unwrap(),
                                                     Color::solid_color(0, 0, 255))
                        .draw(geometry).ok();
                }
            },
            Container::Container { ref mut borders, geometry, .. } => {
                if let Some(borders_) = borders.take() {
                    *borders = SimpleDraw::new(borders_.enable_cairo().unwrap(),
                                               Color::solid_color(0, 0, 255))
                        .draw(geometry).ok();
                }
            },
            _ => panic!("Tried to render a non-view / non-container")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustwlc::*;

    #[test]
    fn can_have_child() {
        let root = ContainerType::Root;
        let output = ContainerType::Output;
        let workspace = ContainerType::Workspace;
        let container = ContainerType::Container;
        let view = ContainerType::View;

        assert!(root.can_have_child(output),         "Root      > output");
        assert!(output.can_have_child(workspace),    "Output    > workspace");
        assert!(workspace.can_have_child(container), "Workspace > container");
        assert!(container.can_have_child(container), "Container > container");
        assert!(container.can_have_child(view),      "Container > view");

        assert!(!root.can_have_child(root),      "! Root > root");
        assert!(!root.can_have_child(workspace), "! Root > workspace");
        assert!(!root.can_have_child(container), "! Root > container");
        assert!(!root.can_have_child(view),      "! Root > view");

        assert!(!output.can_have_child(root),      "! Output > root");
        assert!(!output.can_have_child(output),    "! Output > output");
        assert!(!output.can_have_child(container), "! Output > container");
        assert!(!output.can_have_child(view),      "! Output > view");

        assert!(!workspace.can_have_child(root),      "! Workspace > root");
        assert!(!workspace.can_have_child(output),    "! Workspace > output");
        assert!(!workspace.can_have_child(workspace), "! Workspace > workspace");
        assert!(!workspace.can_have_child(view),      "! Workspace > view");

        assert!(!container.can_have_child(root),      "! Container > root");
        assert!(!container.can_have_child(workspace), "! Container > workspace");
        assert!(!container.can_have_child(output),    "! Container > container");

        assert!(!view.can_have_child(root),      "! View > root");
        assert!(!view.can_have_child(output),    "! View > output");
        assert!(!view.can_have_child(workspace), "! View > workspace");
        assert!(!view.can_have_child(container), "! View > container");
        assert!(!view.can_have_child(view),      "! View > view");
    }

    #[test]
    #[allow(unused_variables)]
    /// Tests set and get geometry
    fn geometry_test() {
        use rustwlc::*;
        let test_geometry1 = Geometry {
            origin: Point { x: 800, y: 600 },
            size: Size { w: 500, h: 500}
        };
        let test_geometry2 = Geometry {
            origin: Point { x: 1024, y: 2048},
            size: Size { w: 500, h: 700}
        };
        let root = Container::new_root();
        assert!(root.get_geometry().is_none());

        let output = Container::new_output(WlcView::root().as_output());

        let workspace = Container::new_workspace("1".to_string(),
                                                     Size { w: 500, h: 500 });
        assert_eq!(workspace.get_geometry(), Some(Geometry {
            size: Size { w: 500, h: 500},
            origin: Point { x: 0, y: 0}
        }));
    }

    #[test]
    fn layout_change_test() {
        let root = Container::new_root();
        let output = Container::new_output(WlcView::root().as_output());
        let workspace = Container::new_workspace("1".to_string(),
                                                     Size { w: 500, h: 500 });
        let mut container = Container::new_container(Geometry {
            origin: Point { x: 0, y: 0},
            size: Size { w: 0, h:0}
        });
        let view = Container::new_view(WlcView::root());

        /* Container first, the only thing we can set the layout on */
        let layout = match container {
            Container::Container { ref layout, .. } => layout.clone(),
            _ => panic!()
        };
        assert_eq!(layout, Layout::Horizontal);
        let layouts = [Layout::Vertical, Layout::Horizontal];
        for new_layout in &layouts {
            container.set_layout(*new_layout).ok();
            let layout = match container {
                Container::Container { ref layout, .. } => layout.clone(),
                _ => panic!()
            };
            assert_eq!(layout, *new_layout);
        }

        for new_layout in &layouts {
            for container in &mut [root.clone(), output.clone(),
                                   workspace.clone(), view.clone()] {
                let result = container.set_layout(*new_layout);
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn floating_tests() {
        let mut root = Container::new_root();
        let mut output = Container::new_output(WlcView::root().as_output());
        let mut workspace = Container::new_workspace("1".to_string(),
                                                 Size { w: 500, h: 500 });
        let mut container = Container::new_container(Geometry {
            origin: Point { x: 0, y: 0},
            size: Size { w: 0, h:0}
        });
        let mut view = Container::new_view(WlcView::root());
        // by default, none are floating.
        assert!(!root.floating());
        assert!(!output.floating());
        assert!(!workspace.floating());
        assert!(!container.floating());
        assert!(!view.floating());

        // trying to do anything to root, output, or workspace is Err.
        assert_eq!(root.set_floating(true),  Err(ContainerType::Root));
        assert_eq!(root.set_floating(false), Err(ContainerType::Root));
        assert_eq!(output.set_floating(true),  Err(ContainerType::Output));
        assert_eq!(output.set_floating(false), Err(ContainerType::Output));
        assert_eq!(workspace.set_floating(true),  Err(ContainerType::Workspace));
        assert_eq!(workspace.set_floating(false), Err(ContainerType::Workspace));

        assert_eq!(container.set_floating(true),  Ok(ContainerType::Container));
        assert!(container.floating());
        assert_eq!(container.set_floating(false), Ok(ContainerType::Container));
        assert!(!container.floating());

        assert_eq!(view.set_floating(true),  Ok(ContainerType::View));
        assert!(view.floating());
        assert_eq!(view.set_floating(false), Ok(ContainerType::View));
        assert!(!view.floating());
    }
}
