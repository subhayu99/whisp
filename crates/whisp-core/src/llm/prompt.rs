/// System prompt for the suggestion LLM.
pub fn system_prompt() -> &'static str {
    "You are a writing assistant. Your task is to suggest improved versions of text \
     the user is currently typing. Be concise, preserve the user's intent and tone, \
     and only suggest if you can meaningfully improve the text. If the text is already \
     good, respond with exactly the same text. Do not add explanations or commentary — \
     output only the improved text."
}

/// Build the user prompt from the typed text and optional surrounding context.
pub fn build_suggestion_prompt(typed_text: &str, context: Option<&str>) -> String {
    match context {
        Some(ctx) => {
            format!(
                "Context (what the user sees on screen):\n{}\n\n\
                 Text being typed:\n{}\n\n\
                 Suggest an improved version:",
                ctx, typed_text
            )
        }
        None => {
            format!(
                "Text being typed:\n{}\n\n\
                 Suggest an improved version:",
                typed_text
            )
        }
    }
}
