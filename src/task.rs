use crate::TaskArgs;
use std::env::current_dir;
use std::fs::ReadDir;
use std::io;
use std::path::Path;
use std::path::PathBuf;
/// Obtain tasks from the current working directory.
pub struct Tasks {
    queue: ReadDir, /*Worker<Option<DirEntry>>*/
    device_num: u8,
    output_dir: PathBuf,
}
impl Tasks {
    /// Creates a new Task.
    pub fn create(args: &TaskArgs) -> io::Result<Self> {
        let cur_dir = current_dir()?;
        Ok(Self {
            queue: Tasks::get_tasks(&cur_dir)?,
            device_num: args.device,
            output_dir: Tasks::create_output_dir(&cur_dir, args.output_dir.as_str()),
        })
    }
    /// Returns a work-stealing queue from which worker threads are going to steal.
    pub fn get_main_worker(self) -> ReadDir /*Worker<Option<DirEntry>>*/ {
        self.queue
    }
    /// Returns the specified amount of worker threads to be used.
    pub fn get_device(&self) -> u8 {
        self.device_num
    }
    /// Returns the output directory
    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
    /// Attempts to calculate the upper limit of the amount of work each thread should take.
    fn get_tasks(cur_dir: &PathBuf) -> io::Result<ReadDir> /*io::Result<Worker<Option<DirEntry>>>*/
    {
        Ok(std::fs::read_dir(cur_dir)?)
    }
    fn create_output_dir(cur_dir: &Path, output_dir: &str) -> PathBuf {
        let output_path = PathBuf::from(output_dir);
        if !cur_dir.join(output_path.as_path()).exists() {
            if let Err(e) = std::fs::create_dir(output_dir) {
                eprintln!("Cannot create output dir {output_dir}\n{e}")
            }
        }
        output_path
    }
}
