use rust_bert::gpt2::{Gpt2ModelResources, Gpt2VocabResources};
use rust_bert::pipelines::conversation::{
    ConversationConfig, ConversationManager, ConversationModel,
};
use rust_bert::RustBertError;
use rust_bert::resources::RemoteResource;
use serenity::prelude::TypeMapKey;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::oneshot;
use tokio::task;

type Message = (String, oneshot::Sender<Option<String>>);

pub struct MLChat {
    pub sender: mpsc::SyncSender<Message>,
}

impl TypeMapKey for MLChat {
    type Value = Arc<MLChat>;
}

impl MLChat {
    fn process(messages: mpsc::Receiver<Message>) -> Result<(), RustBertError> {
        println!(">>= Process thread started: setup");
        let started = Instant::now();
        let model = ConversationModel::new(ConversationConfig {
            ..Default::default()
        })?;
        let mut manager = ConversationManager::new();
        println!(
            ">>= Process thread started: setup done in {:?}",
            started.elapsed()
        );

        while let Ok((msg, sender)) = messages.recv() {
            println!(">>= Processing: {msg}");

            let started = Instant::now();

            let id = manager.create(&msg);
            let output = model.generate_responses(&mut manager);
            let response = output.get(&id).map(|r| r.to_string());
            sender.send(response).ok();
            manager.remove(&id);

            println!(">>= Processing done: '{msg}' in {:?}", started.elapsed());
        }

        Ok(())
    }

    pub fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel(100);
        thread::Builder::new()
            .name("Processing thread".to_string())
            .spawn(move || {
                if let Err(e) = Self::process(receiver) {
                    eprintln!(">>=[error on conversation manager]: {e:?}")
                }
            })
            .ok();

        Self { sender }
    }

    pub async fn ask(&self, text: String) -> Option<String> {
        let (sender, receiver) = oneshot::channel();
        task::block_in_place(|| self.sender.send((text, sender))).ok()?;

        receiver.await.unwrap()
    }
}
