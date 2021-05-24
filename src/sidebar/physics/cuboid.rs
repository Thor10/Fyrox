use crate::{
    gui::{BuildContext, Ui, UiMessage, UiNode},
    physics::Collider,
    scene::commands::{physics::SetCuboidHalfExtentsCommand, SceneCommand},
    send_sync_message,
    sidebar::{make_f32_input_field, make_text_mark, COLUMN_WIDTH, ROW_HEIGHT},
    Message,
};
use rg3d::{
    core::{algebra::Vector3, pool::Handle},
    gui::{
        grid::{Column, GridBuilder, Row},
        message::{MessageDirection, NumericUpDownMessage, UiMessageData},
        widget::WidgetBuilder,
    },
    scene::physics::CuboidDesc,
};
use std::sync::mpsc::Sender;

pub struct CuboidSection {
    pub section: Handle<UiNode>,
    half_width: Handle<UiNode>,
    half_height: Handle<UiNode>,
    half_depth: Handle<UiNode>,
    sender: Sender<Message>,
}

impl CuboidSection {
    pub fn new(ctx: &mut BuildContext, sender: Sender<Message>) -> Self {
        let half_width;
        let half_height;
        let half_depth;
        let section = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(make_text_mark(ctx, "Half Width", 0))
                .with_child({
                    half_width = make_f32_input_field(ctx, 0, 0.0, std::f32::MAX, 0.1);
                    half_width
                })
                .with_child(make_text_mark(ctx, "Half Height", 1))
                .with_child({
                    half_height = make_f32_input_field(ctx, 1, 0.0, std::f32::MAX, 0.1);
                    half_height
                })
                .with_child(make_text_mark(ctx, "Half Depth", 2))
                .with_child({
                    half_depth = make_f32_input_field(ctx, 2, 0.0, std::f32::MAX, 0.1);
                    half_depth
                }),
        )
        .add_column(Column::strict(COLUMN_WIDTH))
        .add_column(Column::stretch())
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .build(ctx);

        Self {
            section,
            sender,
            half_height,
            half_width,
            half_depth,
        }
    }

    pub fn sync_to_model(&mut self, cuboid: &CuboidDesc, ui: &mut Ui) {
        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.half_width,
                MessageDirection::ToWidget,
                cuboid.half_extents.x,
            ),
        );
        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.half_height,
                MessageDirection::ToWidget,
                cuboid.half_extents.y,
            ),
        );
        send_sync_message(
            ui,
            NumericUpDownMessage::value(
                self.half_depth,
                MessageDirection::ToWidget,
                cuboid.half_extents.z,
            ),
        );
    }

    pub fn handle_message(
        &mut self,
        message: &UiMessage,
        cuboid: &CuboidDesc,
        handle: Handle<Collider>,
    ) {
        if let UiMessageData::NumericUpDown(NumericUpDownMessage::Value(value)) = *message.data() {
            if message.direction() == MessageDirection::FromWidget {
                if message.destination() == self.half_width && cuboid.half_extents.x.ne(&value) {
                    self.sender
                        .send(Message::DoSceneCommand(SceneCommand::SetCuboidHalfExtents(
                            SetCuboidHalfExtentsCommand::new(
                                handle,
                                Vector3::new(value, cuboid.half_extents.y, cuboid.half_extents.z),
                            ),
                        )))
                        .unwrap();
                } else if message.destination() == self.half_height
                    && cuboid.half_extents.y.ne(&value)
                {
                    self.sender
                        .send(Message::DoSceneCommand(SceneCommand::SetCuboidHalfExtents(
                            SetCuboidHalfExtentsCommand::new(
                                handle,
                                Vector3::new(cuboid.half_extents.x, value, cuboid.half_extents.z),
                            ),
                        )))
                        .unwrap();
                } else if message.destination() == self.half_depth
                    && cuboid.half_extents.z.ne(&value)
                {
                    self.sender
                        .send(Message::DoSceneCommand(SceneCommand::SetCuboidHalfExtents(
                            SetCuboidHalfExtentsCommand::new(
                                handle,
                                Vector3::new(cuboid.half_extents.x, cuboid.half_extents.y, value),
                            ),
                        )))
                        .unwrap();
                }
            }
        }
    }
}
