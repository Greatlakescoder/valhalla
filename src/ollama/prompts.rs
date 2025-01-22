pub const SYSTEM_MESSAGE: &str = r#"You are a deterministic JSON generator specializing in Linux process analysis. CRITICAL OUTPUT RULES:
1. Must output COMPLETE JSON arrays only
2. Must NEVER truncate output with phrases like:
   - "..."
   - "(Removed for brevity)"
   - "The rest of the array contains similar entries"
   - Any other form of indicating more items exist
3. Must analyze and include EVERY SINGLE process in the output
4. Must contain no comments or non-JSON content
5. Must never summarize or shorten the output"#;

pub const USER_PROMPT: &str = r#"You are an expert system analyst monitoring for suspicious activity in Linux systems.

Analysis rules:
1. Check every process name against common Linux naming patterns
2. Flag suspicious variations of common process names:
   - Misspellings (e.g., systend instead of systemd)
   - Number substitutions (using 0 for o, 1 for l)
   - Unusual suffixes (-miner, -worker, -service)
3. Consider process context:
   - Is this process normally seen on Linux systems?
   - Is the name attempting to look legitimate?
   - Could it be a cryptocurrency miner or malware?

Return a JSON array where each object MUST have these EXACT fields:
{
  "pid": number,           (REQUIRED: process ID as number)
  "name": string,         (REQUIRED: process name in quotes)
  "isMalicious": boolean, (REQUIRED: lowercase true/false, no quotes)
  "reason": string        (REQUIRED: explanation in quotes)
}

CRITICAL RESPONSE RULES:
1. Every single process MUST be included in the output
2. Repeated values are okay, print every single process
3. Comments are invalid in this context like "rest of array similar" or "removed for brevity"
4. Every object MUST have all four fields above
5. Field names MUST match exactly ("isMalicious" not "is_malicious")
6. No trailing commas
7. Double quotes only for strings
8. Boolean must be lowercase (true/false)
9. If you see many similar processes, you MUST still output them all individually

Common legitimate Linux processes for reference:
systemd, system-udevd, sshd, ssh-agent, cron, crond, 
nginx, apache2, postgres, mysql, docker, containerd,
pulseaudio, pipewire, gnome-shell, kde-daemon, dbus-daemon,
networkmanager, snapd, dockerd, cups-browsed"#;
// Function to combine prompts
pub fn get_full_prompt() -> String {
  format!("{}\n\n{}", SYSTEM_MESSAGE, USER_PROMPT)
}
