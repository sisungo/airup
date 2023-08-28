macro_rules! task_feedback_from {
    ($t:ty, $v:tt) => {
        impl From<$t> for TaskFeedback {
            fn from(val: $t) -> Self {
                Self::$v(val)
            }
        }
    };
}

#[derive(Debug, Clone)]
pub enum TaskFeedback {
    Nothing(()),
}
task_feedback_from!((), Nothing);
