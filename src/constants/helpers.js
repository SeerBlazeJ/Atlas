export function dateGroup(iso) {
  const d = new Date(iso);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const dd = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  if (dd.getTime() === today.getTime()) return "Today";
  const yesterday = new Date(today - 86400000);
  if (dd.getTime() === yesterday.getTime()) return "Yesterday";
  if (dd > new Date(today - 7 * 86400000)) return "Previous 7 Days";
  return "Older";
}
