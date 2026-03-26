use relm4::{ComponentSender, Worker};
use std::sync::Arc;

use crate::features::core::module_engine::{
    sword_engine::SwordEngine, sword_engine_books_and_chapter_ext::CategorizedBook,
    sword_engine_module_content_ext::Section, sword_module::SwordModule,
};

pub struct BibleWorker {
    engine: Arc<SwordEngine>,
}

#[derive(Debug)]
pub enum BibleWorkerInput {
    LoadChapter {
        module: SwordModule,
        reference: String,
    },
    GetBooks {
        module_name: String,
    },
    GetBookName {
        module_name: String,
        book_index: usize,
        chapter: i32,
    },
}

#[derive(Debug)]
pub enum BibleWorkerOutput {
    ChapterLoaded(Vec<Section>),
    BooksLoaded(Vec<CategorizedBook>),
    BookNameLoaded {
        name: String,
        chapter: i32,
        chapter_count: i32,
    },
}

impl Worker for BibleWorker {
    type Init = ();
    type Input = BibleWorkerInput;
    type Output = BibleWorkerOutput;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            // Assuming SwordEngine::new() returns Arc<SwordEngine>
            // or you wrap it here.
            engine: SwordEngine::new(),
        }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        let engine = self.engine.clone();

        match msg {
            BibleWorkerInput::LoadChapter { module, reference } => {
                std::thread::spawn(move || {
                    let sections = engine.get_whole_chapter(&module, &reference);
                    let _ = sender.output(BibleWorkerOutput::ChapterLoaded(sections));
                });
            }

            BibleWorkerInput::GetBooks { module_name } => {
                std::thread::spawn(move || {
                    // Offloading the heavy categorized books lookup
                    let books = engine.get_categorized_books(&module_name);
                    let _ = sender.output(BibleWorkerOutput::BooksLoaded(books));
                });
            }

            BibleWorkerInput::GetBookName {
                module_name,
                book_index,
                chapter,
            } => {
                std::thread::spawn(move || {
                    let chapter_count = engine.get_chapter_count(&module_name, book_index);
                    let name = engine.get_book_name(&module_name, book_index);
                    let _ = sender.output(BibleWorkerOutput::BookNameLoaded {
                        name,
                        chapter,
                        chapter_count,
                    });
                });
            }
        }
    }
}
