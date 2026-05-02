use crate::config::log_options::LogOptions;

pub(crate) trait WeiduInstallBlock {
  fn generate_weidu_args(
    &self,
    weidu_log_mode: Vec<LogOptions>,
    language: &str,
    generic_weidu_args: &[String],
  ) -> Vec<String>;
  fn log_file_name(&self) -> String;
}
