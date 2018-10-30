use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};

pub fn theme_option(
    h: &Helper,
    _r: &Handlebars,
    ctx: &Context,
    rc: &mut RenderContext,
    out: &mut Output,
) -> Result<(), RenderError> {
    trace!("theme_option (handlebars helper)");

    let param = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::new(
            "Param 0 with String type is required for theme_option helper.",
        ))?;

    let theme_name = rc
        .evaluate_absolute(ctx, "default_theme", true)?
        .as_str()
        .ok_or_else(|| RenderError::new("Type error for `default_theme`, string expected"))?;

    out.write(param)?;
    if param.to_lowercase() == theme_name.to_lowercase() {
        out.write(" (default)")?;
    }

    Ok(())
}
