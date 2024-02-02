mod error;

pub mod file_system;

pub mod tvc;
mod js_vec;
mod vscode;
mod directory;
mod sleep;
mod discussion;

#[cfg(test)]
pub mod tests {
    use crate::directory::home_dir;
    use crate::file_system::node::NodeFileSystem;

    pub fn workspace_folder() -> String {
        format!("{}/tmp", home_dir())
    }

    pub fn node_fs() -> NodeFileSystem {
        NodeFileSystem::new(workspace_folder())
    }
}
