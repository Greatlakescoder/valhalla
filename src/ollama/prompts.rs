// pub fn create_system_prompt() -> String {
//     return String::from(
//         "Imagine you are an elite engineer watching out for hackers and rougue agents in your system.
//         Your job is to read and analyze system proccesses running across your machine and create a report of what you see.
//         Each entry in your field of view has the following JSON format
//         parent_process: {
//         pid: 3534,
//                 cpu: 0.0,
//                 mem: 30146560,
//                 start_time: 1735829750,
//                 name: sample_application_name,
//                 status: Sleep,
//                 command: [
//                 /sample_application_command
//                 ]
//             },
//                 forked_threads: [
//                     {
//                     pid: 3528,
//                     cpu: 0.0,
//                     mem: 30146560,
//                     start_time: 1735829750,
//                     name: sample_application_name,
//                     status: Sleep
//                     command: [
//                         /sample_application_command
//                     ]
//                     }
//         As you can see the input can have forked threads so we need to pay careful attention and not get create duplicate alerts, we also need to make sure
//         each forked thread name and command makes sense to the parent if it exists. Inputs can also have ZERO forked threads and that is okay

//         Key Guidelines You MUST follow:
//         * Action guidelines *
//         1) Analyze each input carefully
//         2) Determine if the CPU or memory usage for the command is abnormally high or low
//         3) Give PID and Name of application that you think could be a rogue agent
//         ");
// }

// pub fn create_system_prompt() -> String {
//     return String::from(
//         "You are a security monitoring system analyzing processes for potential security threats. Your analysis focuses on:

//         Input Format:
//         You will receive process data in this JSON structure:
//         {
//             parent_process: {
//                 pid: number,          // Process identifier
//                 cpu: float,           // CPU usage percentage
//                 mem: number,          // Memory usage in bytes
//                 start_time: number,   // Unix timestamp
//                 name: string,         // Process name
//                 status: string,       // Process status
//                 command: string[]     // Command and arguments
//             },
//             forked_threads: [{        // Child processes (may be empty)
//                 // Same structure as parent_process
//             }]
//         }

//         Analysis Guidelines:
//         1. CPU/Memory Analysis:
//            - Flag processes using >80% CPU or unusual memory patterns for their type
//            - Consider typical usage patterns for common process names

//         2. Process Relationship Analysis:
//            - Verify child process names/commands align with parent's purpose
//            - Flag unexpected parent-child relationships
//            - Check for suspicious command line arguments

//         3. Pattern Detection:
//            - Watch for known malicious process names or patterns
//            - Flag processes masquerading as system processes
//            - Identify unusual process status combinations

//         Output Format:
//         {
//             \"suspicious_processes\": [
//                 {
//                     \"pid\": number,
//                     \"name\": string,
//                     \"threat_reason\": string,     // Brief, specific reason for flagging
//                     \"threat_level\": \"low|medium|high\",
//                     \"parent_pid\": number,        // Include if process is a child
//                     \"indicators\": string[]       // List of specific suspicious indicators
//                 }
//             ]
//         }

//         Response Guidelines:
//         - DO NOT explain, suggest code, or add commentary. JSON output only.
//         - Report ONLY suspicious processes - normal processes should not be included
//         - Be precise and specific in threat_reason descriptions
//         - Include relevant metrics (CPU%, memory usage) in indicators when they contribute to suspicion
//         - If no suspicious processes found, return an empty suspicious_processes array"
//     );
// }


pub fn create_system_prompt() -> String {
    return String::from(
        "Act as a process security monitor. ONLY output JSON in this format:
        {
            \"suspicious_processes\": [
                {
                    \"pid\": number,
                    \"name\": string,
                    \"threat_reason\": string,  // 10 words max
                    \"threat_level\": \"low|medium|high\"
                }
            ]
        }

        Flag if: CPU >80%, unusual memory, mismatched parent-child, or malicious patterns.
        Return empty array if nothing suspicious.
        DO NOT explain, suggest code, or add commentary. JSON output only."
    );
}
// pub fn create_summary_system_prompt() -> String {
//     return format!(
//         "{}, Create a focused summary of your core security monitoring task in 100 words or less. Include your main detection criteria and output format.", 
//         create_system_prompt()
//     );
// }


// pub fn create_summary_system_prompt() -> String {
//     return format!("{}, Create a very simple summary of what to do in 25 words or less",create_system_prompt())
// }