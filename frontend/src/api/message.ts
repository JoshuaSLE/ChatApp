import api from "./client";

import type {
  CreateMessage,
  MessageResponse,
  GetMessage,
  GetMessageResponse,
} from "../types/messages";

export const create = (room_id: string, data: CreateMessage) =>
  api.post<MessageResponse>(`/rooms/${room_id}/messages/`, data);

export const get = (room_id: string, params: GetMessage) =>
  api.post<GetMessageResponse[]>(`/rooms/${room_id}/messages/`, { params });
