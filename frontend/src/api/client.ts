import axios from "axios";

export const api = axios.create({
  baseURL: "/api",
  withCredentials: true,
});

let accessToken: string | null = null;
export function setAccessToken(token: string | null) {
  accessToken = token;
}

api.interceptors.request.use((config) => {
  if (accessToken) config.headers.Authorization = `Bearer ${accessToken}`;
  return config;
});

let isRefreshing = false;
let queue: Array<() => void> = [];

api.interceptors.response.use(
  (res) => res,
  async (error) => {
    const original = error.config;
    const isRefreshCall = original?.url?.includes("/auth/refresh");

    if (error.response?.status === 401 && !original._retry && !isRefreshCall) {
      original._retry = true;

      if (isRefreshing) {
        return new Promise((resolve) =>
          queue.push(() => resolve(api(original))),
        );
      }

      isRefreshing = true;
      try {
        const { data } = await api.post("/auth/refresh");
        setAccessToken(data.access_token);
        queue.forEach((run) => run());
        queue = [];
        return api(original);
      } catch (refreshError) {
        queue = [];
        setAccessToken(null);
        window.dispatchEvent(new Event("auth:logout"));

        return Promise.reject(refreshError);
      } finally {
        isRefreshing = false;
      }
    }
    return Promise.reject(error);
  },
);

export default api;
