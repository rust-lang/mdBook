use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use log::trace;

pub fn theme_option(
    h: &Helper<'_, '_>,
    _r: &Handlebars<'_>,
    ctx: &Context,
    rc: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    trace!("theme_option (handlebars helper)");

    let param = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        RenderError::new("Param 0 with String type is required for theme_option helper.")
    })?;

    let default_theme = rc.evaluate(ctx, "@root/default_theme")?;
    let default_theme_name = default_theme
        .as_json()
        .as_str()
        .ok_or_else(|| RenderError::new("Type error for `default_theme`, string expected"))?;

    out.write(param)?;
    if param.to_lowercase() == default_theme_name.to_lowercase() {
        out.write(" (default)")?;
    }

    Ok(())
}
