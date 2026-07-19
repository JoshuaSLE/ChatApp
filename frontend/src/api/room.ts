import type {
  CreateRoom,
  CreateRoomResponse,
  ListRoomResponse,
  MeRoomResponse,
  UpdateRoom,
  UpdateRoomResponse,
} from "../types/rooms";
import api from "./client";

export const create = (data: CreateRoom) =>
  api.post<CreateRoomResponse>("/rooms/", data);

export const list = () => api.get<ListRoomResponse[]>("/rooms/");

export const update = (room_id: string, data: UpdateRoom) =>
  api.patch<UpdateRoomResponse>(`/rooms/update/${room_id}`, data);

export const remove = (room_id: string) =>
  api.delete(`/rooms/delete/${room_id}`);

export const me = (room_id: string) =>
  api.get<MeRoomResponse>(`/rooms/me/${room_id}`);
