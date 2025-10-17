use bitflags::bitflags;

bitflags! {
    pub struct Permission: i32 {
        // 超级管理员标志位（最高位）
        const ROOT       = 0b10000000_00000000_00000000_00000000; // 1 << 31

        // 资源 (7位: 24-30)
        // const RES_POST   = 0b00000001_00000000_00000000_00000000; // 1 << 24
        const RES_USER   = 0b00000010_00000000_00000000_00000000; // 2 << 24
        // const RES_FILE   = 0b00000011_00000000_00000000_00000000; // 3 << 24

        // 操作 (8位: 16-23)
        const ACT_READ   = 0b00000000_00000001_00000000_00000000; // 1 << 16
        const ACT_WRITE  = 0b00000000_00000010_00000000_00000000; // 2 << 16
        const ACT_DELETE = 0b00000000_00000100_00000000_00000000; // 4 << 16

        // 所有权 (1位: 15)
        const OWN        = 0b00000000_00000000_10000000_00000000; // 1 << 15
    }
}

impl Permission {
    pub fn get_roles(&self) -> Vec<Role> {
        let all_roles = [Role::Root, Role::Admin, Role::User, Role::Guest];
        all_roles
            .iter()
            .filter(|&&role| self.contains(role.permissions()))
            .cloned()
            .collect()
    }

    pub fn set_role(&mut self, role: Role) {
        *self = role.permissions();
    }

    pub fn add_role(&mut self, role: Role) {
        *self |= role.permissions();
    }

    /// 删除某角色的权限（只移除该角色独有的权限部分）
    pub fn remove_role(&mut self, role: Role) {
        // 1. 获取当前权限对应的所有角色
        let matched_roles = self.get_roles();

        // 2. 计算其他角色的权限并集（排除当前要移除的角色）
        let other_roles_perms = matched_roles
            .iter()
            .filter(|&&r| r != role)
            .fold(Permission::empty(), |acc, &r| acc | r.permissions());

        // 3. 计算目标角色的独有权限
        let role_unique_perms = role.permissions() - other_roles_perms;

        // 4. 仅移除独有权限
        *self &= !role_unique_perms;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    Root,  // 超级管理员
    Admin, // 管理员
    User,  // 普通用户
    Guest, // 访客
}

impl Role {
    /// 为每个角色分配权限
    pub fn permissions(self) -> Permission {
        match self {
            Role::Root => Permission::ROOT, // ROOT 拥有所有权限
            Role::Admin => {
                Permission::RES_USER
                    | Permission::ACT_READ
                    | Permission::ACT_WRITE
                    | Permission::ACT_DELETE
            }
            Role::User => Permission::ACT_READ | Permission::OWN,
            Role::Guest => Permission::ACT_READ, // 仅可读
        }
    }

    pub fn has_permission(self, permission: Permission) -> bool {
        self.permissions().contains(permission)
    }
}
