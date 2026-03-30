pub fn password_reset_email(reset_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<body style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
    <h1 style="color: #333;">Password Reset</h1>
    <p>You requested a password reset. Click the link below to set a new password:</p>
    <p><a href="{reset_url}" style="display: inline-block; padding: 12px 24px;
        background-color: #4f46e5; color: white; text-decoration: none;
        border-radius: 6px;">Reset Password</a></p>
    <p style="color: #666; font-size: 14px;">This link expires in 1 hour.
        If you did not request this reset, you can safely ignore this email.</p>
</body>
</html>"#
    )
}
