use rafflesia_web::compiler_worker::CompilerWorker;
use yew_agent::PublicWorker;

fn main() {
    CompilerWorker::register();
}