export type CreateMessage = {
  readonly body?: string;
  readonly attachment_key?: string;
  readonly attachment_type?: string;
};

export type MessageResponse = {
  readonly body?: string;
  readonly attachment_key?: string;
  readonly attachment_type?: string;
};

export type GetMessage = {
  readonly limit?: number;
  readonly before?: Date;
};

export type GetMessageResponse = {
  readonly id: string;
  readonly username: string;
  readonly body?: string;
  readonly attachment_key?: string;
  readonly attachment_type?: string;
  readonly created_at: Date;
};
