use crate::settings::FormalScene;

pub fn skill_prompt(scene: FormalScene) -> &'static str {
    match scene {
        FormalScene::General => include_str!("general.md"),
        FormalScene::Email => include_str!("email.md"),
        FormalScene::Greeting => include_str!("greeting.md"),
        FormalScene::ProfessionalReply => include_str!("professional_reply.md"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_scene_specific_skill_prompt() {
        assert!(skill_prompt(FormalScene::Email).contains("email body"));
        assert!(skill_prompt(FormalScene::ProfessionalReply).contains("professional reply"));
    }
}
