use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::task::JoinSet;

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct EventRegistration<T: std::marker::Sync>(Vec<BedrockEventConfig<T>>);

impl<T: std::marker::Sync> EventRegistration<T> {
    pub fn validate(&self) -> bool {
        self.0.iter().all(BedrockEventConfig::file_exists)
    }
    pub async fn read_all(&self) -> Result<Vec<String>, std::io::Error> {
        self.0
            .iter()
            .map(|x| {
                let path = x.file.clone();
                async { tokio::fs::read_to_string(path).await }
            })
            .collect::<JoinSet<_>>()
            .join_all()
            .await
            .into_iter()
            .collect::<Result<Vec<String>, std::io::Error>>()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Deserialize, Serialize)]
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

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnScore {
    name: String,
}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnComplete {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnPause {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnUnpause {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnTestEvaluation {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnSubmissionEvaluation {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnTeamKick {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnTeamBan {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnAnnouncement {}
#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnCheckIn {}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Events {
    #[serde(default)]
    pub on_score: EventRegistration<BedrockEventConfig<OnScore>>,
    #[serde(default)]
    pub on_complete: EventRegistration<BedrockEventConfig<OnComplete>>,
    #[serde(default)]
    pub on_pause: EventRegistration<BedrockEventConfig<OnPause>>,
    #[serde(default)]
    pub on_unpause: EventRegistration<BedrockEventConfig<OnUnpause>>,
    #[serde(default)]
    pub on_test_evaluation: EventRegistration<BedrockEventConfig<OnTestEvaluation>>,
    #[serde(default)]
    pub on_submission_evaluation: EventRegistration<BedrockEventConfig<OnSubmissionEvaluation>>,
    #[serde(default)]
    pub on_team_kick: EventRegistration<BedrockEventConfig<OnTeamKick>>,
    #[serde(default)]
    pub on_team_ban: EventRegistration<BedrockEventConfig<OnTeamBan>>,
    #[serde(default)]
    pub on_announcement: EventRegistration<BedrockEventConfig<OnAnnouncement>>,
    #[serde(default)]
    pub on_check_in: EventRegistration<BedrockEventConfig<OnCheckIn>>,
}

impl Events {
    pub async fn get_all_files(&self) -> Result<Vec<String>, std::io::Error> {
        let mut jset = JoinSet::new();
        jset.spawn({
            let x = self.on_score.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_complete.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_pause.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_unpause.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_test_evaluation.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_submission_evaluation.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_team_kick.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_team_ban.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_announcement.clone();
            async move { x.read_all().await }
        });
        jset.spawn({
            let x = self.on_check_in.clone();
            async move { x.read_all().await }
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
