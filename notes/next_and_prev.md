## Signals messages

The messages defined in signals::Messages are pretty heavy weight; they have us
adding / removing signals, zooming in, out and going next / prev.

### next and prev sm 

For next and prev messages, we should have a state machine that goes as follows:

Message::Next or Message::Prev -> Creates command that gets the next time ->
Command emits a time -> Message::UpdateCursor is emitted -> batched command that
updates the signal values in the sigwindow is sent out



