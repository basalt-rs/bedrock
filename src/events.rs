use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Hash, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct EventRegistration<T: std::marker::Sync>(Vec<BedrockEventConfig<T>>);

impl<T: std::marker::Sync> EventRegistration<T> {
    pub fn validate(&self) -> bool {
        self.0.iter().all(BedrockEventConfig::file_exists)
    }
    /// Synchronously retrieve file contents and path
    pub fn read(&self) -> Result<Vec<(PathBuf, String)>, std::io::Error> {
        self.0
            .iter()
            .map(|x| {
                let path = x.file.clone();
                std::fs::read_to_string(&path).map(|content| (path, content))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<(PathBuf, String)>, std::io::Error>>()
    }
    #[cfg(feature = "tokio")]
    /// Asynchronously retrieve file contents and path
    pub async fn read_all_async(&self) -> Result<Vec<(PathBuf, String)>, std::io::Error> {
        use tokio::task::JoinSet;
        self.0
            .iter()
            .map(|x| {
                let path = x.file.clone();
                async {
                    tokio::fs::read_to_string(&path)
                        .await
                        .map(|content| (path, content))
                }
            })
            .collect::<JoinSet<_>>()
            .join_all()
            .await
            .into_iter()
            .collect::<Result<Vec<(PathBuf, String)>, std::io::Error>>()
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct BedrockEventConfig<T: std::marker::Sync> {
    pub file: PathBuf,
    #[serde(
        flatten,
        bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>")
    )]
    pub options: T,
}

impl<T: std::marker::Sync> BedrockEventConfig<T> {
    pub async fn read_file_data(&self) -> Result<String, std::io::Error> {
        tokio::fs::read_to_string(&self.file).await
    }

    pub fn file_exists(&self) -> bool {
        self.file.exists()
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnScore {
    name: String,
}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnComplete {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnPause {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnUnpause {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnTestEvaluation {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnSubmissionEvaluation {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnTeamKick {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnTeamBan {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnAnnouncement {}
#[derive(PartialEq, Hash, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnCheckIn {}

#[derive(Clone, Hash, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Events {
    #[serde(default)]
    pub on_score: EventRegistration<OnScore>,
    #[serde(default)]
    pub on_complete: EventRegistration<OnComplete>,
    #[serde(default)]
    pub on_pause: EventRegistration<OnPause>,
    #[serde(default)]
    pub on_unpause: EventRegistration<OnUnpause>,
    #[serde(default)]
    pub on_test_evaluation: EventRegistration<OnTestEvaluation>,
    #[serde(default)]
    pub on_submission_evaluation: EventRegistration<OnSubmissionEvaluation>,
    #[serde(default)]
    pub on_team_kick: EventRegistration<OnTeamKick>,
    #[serde(default)]
    pub on_team_ban: EventRegistration<OnTeamBan>,
    #[serde(default)]
    pub on_announcement: EventRegistration<OnAnnouncement>,
    #[serde(default)]
    pub on_check_in: EventRegistration<OnCheckIn>,
}

impl Events {
    /// Synchronously fetch the contents of all files along with their paths
    pub fn get_all_files(&self) -> Result<Vec<(PathBuf, String)>, std::io::Error> {
        let result = vec![self.on_score.read(), self.on_complete.read()]
            .into_iter()
            .collect::<Result<Vec<Vec<_>>, std::io::Error>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(result)
    }

    #[cfg(feature = "tokio")]
    /// Asynchronously fetch the contents of all files along with their paths
    pub async fn get_all_files_async(&self) -> Result<Vec<(PathBuf, String)>, std::io::Error> {
        use tokio::task::JoinSet;
        let mut jset = JoinSet::new();
        jset.spawn({
            let x = self.on_score.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_complete.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_pause.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_unpause.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_test_evaluation.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_submission_evaluation.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_team_kick.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_team_ban.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_announcement.clone();
            async move { x.read_all_async().await }
        });
        jset.spawn({
            let x = self.on_check_in.clone();
            async move { x.read_all_async().await }
        });
        let data = jset
            .join_all()
            .await
            .into_iter()
            .collect::<Result<Vec<Vec<_>>, std::io::Error>>()?;
        Ok(data.into_iter().flatten().collect())
    }

    pub async fn validate(&self) -> bool {
        self.on_score.0.iter().all(BedrockEventConfig::file_exists)
            && self
                .on_complete
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self.on_pause.0.iter().all(BedrockEventConfig::file_exists)
            && self
                .on_unpause
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self
                .on_test_evaluation
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self
                .on_submission_evaluation
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self
                .on_team_kick
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self
                .on_team_ban
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self
                .on_announcement
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
            && self
                .on_check_in
                .0
                .iter()
                .all(BedrockEventConfig::file_exists)
    }
}
