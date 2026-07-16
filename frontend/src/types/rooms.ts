export type CreateRoom = {
  readonly name?: string;
  readonly direct: boolean;
  readonly members: Array<string>;
};

export type CreateRoomResponse = {
  readonly room_id: string;
};

export type ListRoomResponse = {
  readonly room_id: string;
  readonly is_direct: boolean;
  readonly room_name: string;
};

export type UpdateRoom = {
  readonly name?: string;
  readonly members: Array<string>;
};

export type UpdateRoomResponse = {
  readonly name?: string;
  readonly members: Array<string>;
};

export type MeRoomResponse = {
  readonly name?: string;
  readonly is_direct: boolean;
  readonly members: Array<string>;
};
