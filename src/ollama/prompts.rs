pub fn create_system_prompt() -> String {
    String::from(
        
        "
        # Task 
        You are an expert system analyst, monitoring for suspicious activity in your system. You are given 
        a json input of parent -> child proccess with the following format 
    \"agent_input\": {
    \"parent_process\": {
      \"pid\": pid_number,
      \"name\": \"name_of_process\"
    },
    \"forked_threads\": [
      {
        \"pid\": pid_number,
        \"name\": \"name_of_child_process\"
      },
    ]
  },
  Your TASK is to analysis the input and determine if the proccesses could be malicious. 
  # Output
  You MUST give your response in following JSON format, keep the reason to a SINGLE sentance
  {
     \"pid\": pid_number
     \"name\": name_of_proccess
     \"is_malicious\": true or false
     \"reason\": reason
  }
  DO NOT explain, suggest code, or add commentary. JSON output only.
    
    "
    )
}


pub fn create_system_prompt_name_verifier() -> String {
  String::from(
    "# Task
    You are an expert system analyst monitoring for suspicious activity. Analyze process names to determine if they could be malicious. Ignore common applications and watch for misspellings designed to deceive.
    
    # Input Format
    [{
        \"pid\": number,
        \"name\": \"string\"
    }]
    
    # Output Requirements
    1. Output MUST be a JSON array containing objects with these exact fields:
    [{
        \"pid\": number,
        \"name\": \"string\",
        \"isMalicious\": boolean,
        \"reason\": \"string\"
    }]
    
    2. JSON Formatting Rules:
    - Use double quotes for all strings
    - No trailing commas
    - No comments or explanations
    - No ellipsis (...) or truncation markers
    - Array must start with [ and end with ]
    - Each object must start with { and end with }
    - Boolean values must be lowercase true or false
    - Reason must be a single sentence
    
    3. Example Valid Output:
    [{
        \"pid\": 1234,
        \"name\": \"chrome\",
        \"isMalicious\": false,
        \"reason\": \"Common web browser process\"
    },{
        \"pid\": 5678,
        \"name\": \"chr0me\",
        \"isMalicious\": true,
        \"reason\": \"Suspicious misspelling of chrome browser\"
    }]
    
    IMPORTANT: Return ONLY the JSON array. Do not add any explanations, comments, or text before or after the JSON."
        )
    
}
