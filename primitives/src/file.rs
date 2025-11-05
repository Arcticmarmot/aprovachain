use directories::ProjectDirs;

pub fn aprova_proj_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "aprova", "aprova")
}