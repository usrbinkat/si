import { BehaviorSubject, ReplaySubject, Subject } from "rxjs";
import { WsEvent, WsPayloadKinds } from "@/api/sdf/dal/ws_event";
import {
  eventChangeSetApplied$,
  eventChangeSetCanceled$,
  eventChangeSetCreated$,
  eventChangeSetWritten$,
} from "@/observable/change_set";
import { eventResourceRefreshed$ } from "@/observable/resource";
import { eventCheckedQualifications$ } from "@/observable/qualification";
import { eventDependentValuesUpdated$ } from "@/observable/attribute_value";
import { eventCodeGenerated$ } from "@/observable/code";
import { eventSecretCreated$ } from "@/observable/secret";
import { eventCommandOutput$, eventCommandReturn$ } from "@/observable/command";

const eventMap: {
  [E in WsPayloadKinds["kind"]]:  // eslint-disable-next-line @typescript-eslint/no-explicit-any
    | BehaviorSubject<any>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    | ReplaySubject<any>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    | Subject<any>;
} = {
  ChangeSetCreated: eventChangeSetCreated$,
  ChangeSetApplied: eventChangeSetApplied$,
  ChangeSetCanceled: eventChangeSetCanceled$,
  ChangeSetWritten: eventChangeSetWritten$,
  ResourceRefreshed: eventResourceRefreshed$,
  CodeGenerated: eventCodeGenerated$,
  CheckedQualifications: eventCheckedQualifications$,
  UpdatedDependentValue: eventDependentValuesUpdated$,
  SecretCreated: eventSecretCreated$,
  CommandOutput: eventCommandOutput$,
  CommandReturn: eventCommandReturn$,
};

export function dispatch(wsEvent: WsEvent<WsPayloadKinds>) {
  const obs$ = eventMap[wsEvent.payload.kind];
  obs$.next(wsEvent);
}

export const WsEventService = {
  dispatch,
};
