const ADMIN_ROLES = ['global_admin', 'channel_admin']

export const OPERATOR_ROUTES = ['player', 'media', 'media-asset', 'logging', 'message']

export function isAdmin(role: string): boolean {
    return ADMIN_ROLES.includes(role)
}

export function canAccessRoute(role: string, routeName: string): boolean {
    if (isAdmin(role)) {
        return true
    }

    return OPERATOR_ROUTES.includes(routeName)
}