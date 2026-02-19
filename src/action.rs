use iced::Task;
///! Source: https://github.com/airstrike/iced_receipts/blob/master/src/action.rs
use iced::advanced::graphics::futures::MaybeSend;
use std::fmt;

pub struct Action<I, Message> {
    pub instruction: Option<I>,
    pub task: Task<Message>,
}

impl<I, Message> Action<I, Message> {
    /// Create a new `Action` with no `Instruction` or [`Task`](iced::Task).
    pub fn none() -> Self {
        Self {
            instruction: None,
            task: Task::none(),
        }
    }

    /// Create a new `Action` with an `Instruction` and a [`Task`](iced::Task).
    pub fn new(instruction: I, task: Task<Message>) -> Self {
        Self {
            instruction: Some(instruction),
            task,
        }
    }

    /// Create a new `Action` with an `Instruction` to be handled by some ancestor
    /// component.
    pub fn instruction(instruction: I) -> Self {
        Self {
            instruction: Some(instruction),
            task: Task::none(),
        }
    }

    /// Create a new `Action` with a [`Task`](iced::Task).
    pub fn task(task: Task<Message>) -> Self {
        Self {
            instruction: None,
            task,
        }
    }

    /// Map the message of the `Action`'s [`Task`](iced::Task) to a different type.
    pub fn map<N>(self, f: impl Fn(Message) -> N + MaybeSend + 'static) -> Action<I, N>
    where
        Message: MaybeSend + 'static,
        N: MaybeSend + 'static,
    {
        Action {
            instruction: self.instruction,
            task: self.task.map(f),
        }
    }

    /// Maps the `Instruction` of the `Action` to a different type.
    pub fn map_instruction<N>(self, f: impl Fn(I) -> N + MaybeSend + 'static) -> Action<N, Message>
    where
        I: MaybeSend + 'static,
        N: MaybeSend + 'static,
    {
        Action {
            instruction: self.instruction.map(f),
            task: self.task,
        }
    }

    /// Sets the `Instruction` of an `Action`.
    pub fn with_instruction(mut self, instruction: I) -> Self {
        self.instruction = Some(instruction);
        self
    }

    /// Sets the [`Task`](iced::Task) of an `Action`.
    pub fn with_task(mut self, task: Task<Message>) -> Self {
        self.task = task;
        self
    }
}

impl<Instruction: fmt::Debug, Message> fmt::Debug for Action<Instruction, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("instruction", &self.instruction)
            .finish()
    }
}
