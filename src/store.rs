use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::types::{
    answer::{Answer, AnswerId},
    question::{QuestionId, Question}
};

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>
}

impl Store {
    fn new() -> Self {
        Self {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("Failed to read file.")
    }
}