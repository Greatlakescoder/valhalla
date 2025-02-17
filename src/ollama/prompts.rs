pub const PROCESS_ANALYSIS_PROMPT: &str = r#"Score each process name for potential maliciousness on a scale of 0-100.

Input format will be a JSON array of objects containing:
{
    "pid": number,    // Process ID
    "name": string    // Process name
}

Scoring criteria:
- 0-20: Known legitimate system process (e.g., systemd, cron)
- 21-40: Uncommon but likely legitimate process
- 41-60: Unusual or suspicious name pattern
- 61-80: Highly suspicious patterns (misspellings, number substitutions)
- 81-100: Known malware patterns or extremely suspicious

Suspicion factors to consider:
1. Misspellings of common processes (systend, crontab.exe)
2. Number substitutions (syst3md, cr0n)
3. Suspicious suffixes (-miner, -worker, -hidden)
4. Unusual character usage (.exe on Linux, unusual symbols)
5. Impersonation of system processes
6. Cryptocurrency mining related names

Output format must be a JSON array of objects with EXACTLY these fields:
{
    "pid": number,          // Process ID as number
    "name": string,         // Process name
    "score": number,        // 0-100 suspicion score
    "reason": string        // Explanation of score
}

Required format rules:
1. Every process must be included
2. All fields are required
3. Score must be 0-100
4. Use double quotes for strings
5. No comments or truncation
6. No trailing commas
7. Valid JSON only

Common legitimate processes (baseline 0-20 score):
systemd, udevd, sshd, cron, nginx, apache2, postgres,
mysql, docker, containerd, pulseaudio, pipewire,
gnome-shell, dbus-daemon, networkmanager, snapd"#;


pub const RESOURCE_SYSTEM_MESSAGE: &str = r#"You are a deterministic JSON generator specializing in Linux process resource analysis. CRITICAL OUTPUT RULES:
1. Must output COMPLETE JSON arrays only
2. Must NEVER truncate output with phrases like:
   - "..."
   - "(Removed for brevity)"
   - "Any other form of indicating more items exist"
3. Must analyze and include EVERY SINGLE process in the output
4. Must contain no comments or non-JSON content
5. Must never summarize or shorten the output"#;

pub const RESOURCE_USER_PROMPT: &str = r#"You are an expert system analyst monitoring for suspicious resource usage patterns in Linux systems.

Analysis rules:
1. Check process resource patterns for suspicious behavior:
   - Unusually high CPU usage (>80% sustained)
   - Excessive memory consumption
   - Abnormal I/O patterns (constant disk writes/reads)
   - Network connection counts
2. Consider resource usage context:
   - Is this usage pattern normal for this type of process?
   - Does it match known cryptomining patterns?
   - Are there signs of resource abuse or DOS attempts?
3. Flag suspicious combinations:
   - High CPU with hidden network connections
   - Memory leaks or abnormal growth
   - Excessive disk I/O with encrypted content


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
3. No comments or truncation allowed
4. Every object MUST have all five fields above
5. Field names MUST match exactly
6. No trailing commas
7. Double quotes only for strings
8. Boolean must be lowercase (true/false)

Common legitimate resource patterns for reference:
- Database servers: Steady memory, periodic CPU spikes
- Web servers: Moderate CPU, stable memory usage
- System services: Low-moderate resource usage
- Desktop environments: Variable but predictable usage
- Backup processes: High I/O but low CPU usage"#;



pub fn get_resource_verification_prompt() -> String {
    format!("{}\n\n{}", RESOURCE_SYSTEM_MESSAGE, RESOURCE_USER_PROMPT)
}
