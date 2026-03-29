export function getDesktopFitRowCount(
	itemCount: number | null | undefined,
	columns = 3,
	minimumRows = 2
) {
	const normalizedColumns = Math.max(1, Math.floor(columns));
	const normalizedMinimumRows = Math.max(1, Math.floor(minimumRows));
	const normalizedItemCount = Math.max(1, Math.floor(itemCount ?? 0));

	return Math.max(normalizedMinimumRows, Math.ceil(normalizedItemCount / normalizedColumns));
}
