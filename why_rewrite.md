* Not usable in async environment
    * IO not decoupled
    * RequesterContext and ResponderContext are not Send
* Poor error handling
* Parser does not detect all invalid messages
* Responder doesnt' proceed through a state machine. It processes whatever message it gets.
* Overly verbose.
