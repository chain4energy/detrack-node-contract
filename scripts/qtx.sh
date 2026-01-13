#!/bin/bash

# query_tx_wrapper.sh - Wrapper for c4ed query tx

# Default values
FULL_OUTPUT=false
TXHASH=""
FROM_STDIN=false

# Check if input is coming from stdin (pipe)
if [ ! -t 0 ]; then
    FROM_STDIN=true
fi

# Source environment variables
# Assumes config/.env is in a 'config' subdirectory relative to the script's location or project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/config/.env" ]; then
    source "$SCRIPT_DIR/config/.env"
elif [ -f "config/.env" ]; then # If run from a directory containing 'config/.env'
    source "config/.env"
else
    echo "Warning: config/.env not found. Required variables like C4E_CHAIN_ID or HOME_DIR might be missing if not already in environment." >&2
fi

# Parse arguments or read from stdin
if [ "$FROM_STDIN" = true ]; then
    # Read all input from stdin
    STDIN_INPUT=$(cat)
    
    # Parse txhash from the input - look for pattern "txhash: <hash>"
    PARSED_TXHASH=$(echo "$STDIN_INPUT" | grep -oP '(?<=txhash: )[A-Fa-f0-9]{64}' | head -1)
    
    if [ -n "$PARSED_TXHASH" ]; then
        TXHASH="$PARSED_TXHASH"
        echo "$STDIN_INPUT" >&2
        echo "Results of txhash: $TXHASH" >&2
    else
        echo "Error: Could not parse txhash from stdin input." >&2
        echo "Expected format containing 'txhash: <64-character-hex-hash>'" >&2
        exit 1
    fi
    
    # Parse arguments for --full flag
    while [[ "$#" -gt 0 ]]; do
        case $1 in
            -f|--full) FULL_OUTPUT=true; shift ;;
            *)
                echo "Unknown parameter when reading from stdin: $1" >&2
                echo "Usage: <command> | $0 [-f|--full]" >&2
                exit 1
                ;;
        esac
    done
else
    # Original argument parsing for direct usage
    while [[ "$#" -gt 0 ]]; do
        case $1 in
            -f|--full) FULL_OUTPUT=true; shift ;;
            *)
                if [ -z "$TXHASH" ]; then
                    TXHASH="$1"
                else
                    echo "Unknown or duplicate parameter passed: $1" >&2
                    echo "Usage: $0 <txhash> [-f|--full]" >&2
                    exit 1
                fi
                shift ;;
        esac
    done
fi

if [ -z "$TXHASH" ]; then
    if [ "$FROM_STDIN" = true ]; then
        echo "Usage: <command> | $0 [-f|--full]" >&2
        echo "  Input should contain 'txhash: <64-character-hex-hash>'" >&2
    else
        echo "Usage: $0 <txhash> [-f|--full]" >&2
        echo "  <txhash>: The transaction hash to query." >&2
        echo "  -f, --full: Display the full JSON output." >&2
    fi
    exit 1
fi

# Ensure C4E_CHAIN_ID and HOME_DIR are set
if [ -z "$C4E_CHAIN_ID" ]; then
    echo "Error: C4E_CHAIN_ID is not set. Please ensure it's in config/.env or your environment." >&2
    exit 1
fi
if [ -z "$HOME_DIR" ]; then
    # Try to default HOME_DIR if not set, common for Cosmos SDK clients
    if [ -d "$HOME/.c4ed" ]; then
        HOME_DIR="$HOME/.c4ed"
        echo "Warning: HOME_DIR not set, defaulting to $HOME_DIR" >&2
    else
        echo "Error: HOME_DIR is not set for c4ed. Please ensure it's in config/.env, your environment, or $HOME/.c4ed exists." >&2
        exit 1
    fi
fi

# Execute the command and capture output
STDERR_FILE=$(mktemp)
TX_QUERY_RESULT=$(c4ed --home "$HOME_DIR" query tx "$TXHASH" --node "$C4E_RPC_ENDPOINT" --chain-id "$C4E_CHAIN_ID" -o json 2> "$STDERR_FILE")
EXIT_CODE=$?

STDERR_OUTPUT=$(cat "$STDERR_FILE")
rm "$STDERR_FILE"

# Handle c4ed command execution failure
if [ $EXIT_CODE -ne 0 ]; then
    echo "Error: c4ed command failed." >&2
    echo "Exit Code: $EXIT_CODE" >&2
    if [ -n "$STDERR_OUTPUT" ]; then
        echo "Stderr: $STDERR_OUTPUT" >&2
    fi
    if [ -n "$TX_QUERY_RESULT" ]; then # Sometimes c4ed prints non-JSON errors to stdout
        echo "Stdout: $TX_QUERY_RESULT" >&2
    fi
    exit $EXIT_CODE
fi

# Validate JSON output
if ! echo "$TX_QUERY_RESULT" | jq empty 2>/dev/null; then
    echo "Error: c4ed command succeeded (exit code 0) but output was not valid JSON." >&2
    echo "Raw output from c4ed:" >&2
    echo "$TX_QUERY_RESULT" >&2
    if [ -n "$STDERR_OUTPUT" ]; then # Should be empty if exit code was 0
        echo "Stderr output from c4ed (unexpected):" >&2
        echo "$STDERR_OUTPUT" >&2
    fi
    exit 1
fi

# Check for transaction-level errors within the JSON response
JSON_ERROR_CODE=$(echo "$TX_QUERY_RESULT" | jq -r '.code // 0') # Default to 0 if .code is null or not present
JSON_RAW_LOG=$(echo "$TX_QUERY_RESULT" | jq -r '.raw_log // ""')

if [ "$JSON_ERROR_CODE" -ne 0 ] && [ "$JSON_ERROR_CODE" != "null" ]; then
    echo "Error: Transaction query reported an error." >&2
    echo "  Tx Hash: $TXHASH" >&2
    echo "  Error Code: $JSON_ERROR_CODE" >&2
    echo "  Codespace: $(echo "$TX_QUERY_RESULT" | jq -r '.codespace // "N/A"')" >&2
    echo "  Raw Log: $JSON_RAW_LOG" >&2
    
    # Optionally print full JSON if it's an error and --full was requested
    if [ "$FULL_OUTPUT" = true ]; then
        echo "Full JSON response:"
        echo "$TX_QUERY_RESULT"
    fi
    # Try to exit with the numeric error code from JSON if possible
    if [[ "$JSON_ERROR_CODE" =~ ^[0-9]+$ ]]; then
        exit "$JSON_ERROR_CODE"
    else
        exit 1 # Default error exit code
    fi
fi

# Success: c4ed exited 0 and JSON response does not indicate a top-level error
if [ "$FULL_OUTPUT" = true ]; then
    echo "$TX_QUERY_RESULT" | jq --color-output | more
else
    HEIGHT=$(echo "$TX_QUERY_RESULT" | jq -r '.height // "N/A"')
    GAS_WANTED=$(echo "$TX_QUERY_RESULT" | jq -r '.gas_wanted // "N/A"')
    GAS_USED=$(echo "$TX_QUERY_RESULT" | jq -r '.gas_used // "N/A"')
    RAW_LOG_CONTENT=${JSON_RAW_LOG:-"N/A"} # Already defined earlier

    # Construct JSON output using jq
    jq -n \
      --arg txhash "$TXHASH" \
      --arg height "$HEIGHT" \
      --arg gas_wanted "$GAS_WANTED" \
      --arg gas_used "$GAS_USED" \
      --arg status "Success (according to chain query)" \
      '{tx_hash: $txhash, height: $height, gas_wanted: $gas_wanted, gas_used: $gas_used, status: $status}'
fi

exit 0
