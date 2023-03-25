export enum MIDIMessageType {
  NoteOn = 0,
  NoteOff = 1,
  Controller = 3,
}

/**
 * This function is injected by the host. Note that v1,v2,v3 change depending on the kind of event
 * TODO: some wrappers could help here.
 *
 * @param messageType a number representing the type of message to send
 * @param v1 a number representing the first value of the event
 * @param v2 a number representing the second value of the event
 * @param v3 a number representing the third value of the event
 * @param delayMs a number representing when to send the message. can be 0 for immediate sends
 */
declare function $sendLater(
  messageType: MIDIMessageType,
  v1: i32,
  v2: i32,
  v3: i32,
  delayMs: i32
): void;

/**
 * Handles tick events, which are your choosing of tempo subdivision
 * This function should exist in your module for this to work
 */
export function onTick(): void {
  // Do something interesting on your selected subdivision.
  // Or don't. IDK I'm not your mom
}

/**
 * Handles message events, like NoteOn, NoteOff, etc.
 * This function should exist in your module for this to work
 */
export function onMessage(
  messageType: MIDIMessageType,
  v1: i32,
  v2: i32,
  v3: i32
): void {
  switch (messageType) {
    case MIDIMessageType.NoteOn:
    case MIDIMessageType.NoteOff:
      // Take the existing note and send a chord in 500ms
      $sendLater(messageType, v1, v2 - 2, v3, 500);
      $sendLater(messageType, v1, v2, v3, 500);
      $sendLater(messageType, v1, v2 + 2, v3, 500);
      break;

    case MIDIMessageType.Controller:
      // If the controller message comes on channel 0, map it to channel 5
      if (v1 === 0) $sendLater(messageType, 5, v2, v3, 0);
      break;

    default:
      // This is important if you wish to keep other events, they should always be sent
      $sendLater(messageType, v1, v2, v3, 0);
  }
}
