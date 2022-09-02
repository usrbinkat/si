import Bottle from "bottlejs";
import { combineLatest, Observable, take, shareReplay } from "rxjs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";
import { ApiResponse, SDF } from "@/api/sdf";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { DiagramSchemaVariants } from "@/api/sdf/dal/diagram";

export type ListSchemaVariantsRequest = Visibility;
export type ListSchemaVariantsResponse = DiagramSchemaVariants;

export function listSchemaVariants(): Observable<
  ApiResponse<ListSchemaVariantsResponse>
> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return combineLatest([standardVisibilityTriggers$]).pipe(
    take(1),
    switchMap(([[visibility]]) => {
      const request: ListSchemaVariantsRequest = {
        ...visibility,
      };
      return sdf.get<ApiResponse<ListSchemaVariantsResponse>>(
        "diagram/list_schema_variants",
        request,
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
}