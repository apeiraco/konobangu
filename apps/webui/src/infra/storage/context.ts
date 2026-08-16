import {
  LocalStorageService,
  SessionStorageService,
} from "./web-storage.service";

export function provideStorages() {
  return [
    {
      provide: LocalStorageService,
      useFactory: () => new LocalStorageService(),
      deps: [],
    },
    {
      provide: SessionStorageService,
      useFactory: () => new SessionStorageService(),
      deps: [],
    },
  ];
}
