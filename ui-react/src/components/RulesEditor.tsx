import { useState, useEffect } from 'react';
import { X, Plus, Save, Trash2, Download, Upload, Search, Filter } from 'lucide-react';
import { api } from '@/utils/api';
import { useToast } from '@/contexts/ToastContext';
import type { Rule, RuleType, RuleTarget } from '@/types';

interface RulesEditorProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function RulesEditor({ isOpen, onClose }: RulesEditorProps) {
  const [rules, setRules] = useState<Rule[]>([]);
  const [filteredRules, setFilteredRules] = useState<Rule[]>([]);
  const [loading, setLoading] = useState(false);
  const toast = useToast();

  // 表单状态
  const [ruleType, setRuleType] = useState<RuleType>('DOMAIN-SUFFIX');
  const [content, setContent] = useState('');
  const [target, setTarget] = useState<RuleTarget>('PROXY');

  // 搜索和过滤状态
  const [searchText, setSearchText] = useState('');
  const [filterType, setFilterType] = useState<string>('all');
  const [filterTarget, setFilterTarget] = useState<string>('all');

  const loadRules = async () => {
    setLoading(true);
    const result = await api.getRules();
    if (result.success && result.data) {
      setRules(result.data);
      setFilteredRules(result.data);
    }
    setLoading(false);
  };

  useEffect(() => {
    if (isOpen) {
      loadRules();
    }
  }, [isOpen]);

  // 搜索和过滤效果
  useEffect(() => {
    let filtered = [...rules];

    // 按类型过滤
    if (filterType !== 'all') {
      filtered = filtered.filter(rule => rule.rule_type === filterType);
    }

    // 按目标过滤
    if (filterTarget !== 'all') {
      filtered = filtered.filter(rule => rule.target === filterTarget);
    }

    // 按搜索文本过滤
    if (searchText.trim()) {
      const search = searchText.toLowerCase();
      filtered = filtered.filter(rule =>
        rule.content.toLowerCase().includes(search) ||
        rule.rule_type.toLowerCase().includes(search) ||
        rule.target.toLowerCase().includes(search)
      );
    }

    setFilteredRules(filtered);
  }, [rules, searchText, filterType, filterTarget]);

  const handleAdd = async () => {
    if (!content.trim()) {
      toast.warning('请输入规则内容');
      return;
    }

    const rule = `${ruleType},${content},${target}`;
    const result = await api.addRule(rule);
    if (result.success) {
      setContent('');
      await loadRules();
      toast.success('规则已添加');
    } else {
      toast.error(result.error || '添加失败');
    }
  };

  const handleDelete = async (index: number) => {
    if (!confirm('确定删除这条规则？')) return;

    const result = await api.deleteRule(index);
    if (result.success) {
      await loadRules();
      toast.success('规则已删除');
    } else {
      toast.error(result.error || '删除失败');
    }
  };

  const handleSave = async () => {
    const result = await api.saveConfig();
    if (result.success) {
      toast.success('规则已保存到配置文件');
    } else {
      toast.error(result.error || '保存失败');
    }
  };

  // 导出规则
  const handleExport = () => {
    if (rules.length === 0) {
      toast.warning('没有可导出的规则');
      return;
    }

    const rulesText = rules.map(rule => `${rule.rule_type},${rule.content},${rule.target}`).join('\n');
    const blob = new Blob([rulesText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `cat_proxy_rules_${new Date().getTime()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    toast.success('规则已导出');
  };

  // 导入规则
  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.txt,.conf';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = async (event) => {
        const text = event.target?.result as string;
        const lines = text.split('\n').filter(line => line.trim() && !line.startsWith('#'));

        let importedCount = 0;
        for (const line of lines) {
          const parts = line.trim().split(',');
          if (parts.length === 3) {
            const rule = line.trim();
            const result = await api.addRule(rule);
            if (result.success) {
              importedCount++;
            }
          }
        }

        await loadRules();
        toast.success(`成功导入 ${importedCount} 条规则`);
      };
      reader.readAsText(file);
    };
    input.click();
  };

  // 清空所有规则
  const handleClearAll = async () => {
    if (!confirm('确定要清空所有规则吗？此操作不可恢复！')) return;

    for (let i = rules.length - 1; i >= 0; i--) {
      await api.deleteRule(i);
    }

    await loadRules();
    toast.success('所有规则已清空');
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-6 pb-4 border-b border-gray-200 dark:border-gray-700">
          <div>
            <h2 className="text-2xl font-bold text-primary-500">规则管理</h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              共 {rules.length} 条规则 {searchText || filterType !== 'all' || filterTarget !== 'all' ? `(显示 ${filteredRules.length} 条)` : ''}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleImport}
              className="px-3 py-1.5 bg-blue-500 hover:bg-blue-600 text-white rounded-lg text-sm flex items-center gap-1 transition-colors"
            >
              <Upload size={16} />
              导入
            </button>
            <button
              onClick={handleExport}
              className="px-3 py-1.5 bg-green-500 hover:bg-green-600 text-white rounded-lg text-sm flex items-center gap-1 transition-colors"
            >
              <Download size={16} />
              导出
            </button>
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
            >
              <X size={24} />
            </button>
          </div>
        </div>

        {/* 搜索和过滤 */}
        <div className="mb-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchText}
                onChange={(e) => setSearchText(e.target.value)}
                placeholder="搜索规则内容..."
                className="w-full pl-10 pr-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
            <div className="relative">
              <Filter className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="w-full pl-10 pr-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="all">所有类型</option>
                <option value="DOMAIN-SUFFIX">DOMAIN-SUFFIX</option>
                <option value="DOMAIN">DOMAIN</option>
                <option value="DOMAIN-KEYWORD">DOMAIN-KEYWORD</option>
                <option value="IP-CIDR">IP-CIDR</option>
                <option value="GEOIP">GEOIP</option>
                <option value="MATCH">MATCH</option>
              </select>
            </div>
            <div>
              <select
                value={filterTarget}
                onChange={(e) => setFilterTarget(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="all">所有目标</option>
                <option value="DIRECT">DIRECT</option>
                <option value="PROXY">PROXY</option>
                <option value="REJECT">REJECT</option>
              </select>
            </div>
          </div>
          {(searchText || filterType !== 'all' || filterTarget !== 'all') && (
            <div className="mt-2 flex items-center gap-2">
              <button
                onClick={() => {
                  setSearchText('');
                  setFilterType('all');
                  setFilterTarget('all');
                }}
                className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
              >
                清除筛选
              </button>
            </div>
          )}
        </div>

        {/* 规则列表 */}
        <div className="mb-6 max-h-96 overflow-y-auto">
          {loading ? (
            <div className="text-center py-8 text-gray-500">加载中...</div>
          ) : filteredRules.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              {rules.length === 0 ? '暂无规则' : '没有符合条件的规则'}
            </div>
          ) : (
            <div className="space-y-2">
              {filteredRules.map((rule) => {
                // 找到原始索引
                const originalIndex = rules.findIndex(r =>
                  r.rule_type === rule.rule_type &&
                  r.content === rule.content &&
                  r.target === rule.target
                );
                return (
                  <div
                    key={originalIndex}
                    className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                  >
                    <div className="flex items-center gap-3 flex-1">
                      <span className="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded-full font-medium">
                        {rule.rule_type}
                      </span>
                      <span className="font-mono text-sm flex-1">
                        {rule.content}
                      </span>
                      <span className={`px-2 py-1 text-xs rounded-full font-medium ${
                        rule.target === 'PROXY' ? 'bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200' :
                        rule.target === 'DIRECT' ? 'bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200' :
                        'bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-200'
                      }`}>
                        {rule.target}
                      </span>
                    </div>
                    <button
                      onClick={() => handleDelete(originalIndex)}
                      className="p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                    >
                      <Trash2 size={18} />
                    </button>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* 添加规则表单 */}
        <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
          <h3 className="text-lg font-semibold mb-4">添加新规则</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">规则类型</label>
              <select
                value={ruleType}
                onChange={(e) => setRuleType(e.target.value as RuleType)}
                className="select"
              >
                <option value="DOMAIN-SUFFIX">DOMAIN-SUFFIX (域名后缀)</option>
                <option value="DOMAIN">DOMAIN (完整域名)</option>
                <option value="DOMAIN-KEYWORD">DOMAIN-KEYWORD (域名关键字)</option>
                <option value="IP-CIDR">IP-CIDR (IP 段)</option>
                <option value="GEOIP">GEOIP (地理位置)</option>
                <option value="MATCH">MATCH (匹配所有)</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">规则内容</label>
              <input
                type="text"
                value={content}
                onChange={(e) => setContent(e.target.value)}
                placeholder="例如: google.com"
                className="input"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">目标</label>
              <select
                value={target}
                onChange={(e) => setTarget(e.target.value as RuleTarget)}
                className="select"
              >
                <option value="DIRECT">DIRECT (直连)</option>
                <option value="PROXY">PROXY (代理)</option>
                <option value="REJECT">REJECT (拒绝)</option>
              </select>
            </div>

            <div className="flex gap-3 flex-wrap">
              <button onClick={handleAdd} className="btn btn-primary flex items-center gap-2">
                <Plus size={18} />
                添加规则
              </button>
              <button onClick={handleSave} className="btn btn-secondary flex items-center gap-2">
                <Save size={18} />
                保存到文件
              </button>
              {rules.length > 0 && (
                <button
                  onClick={handleClearAll}
                  className="btn bg-red-500 hover:bg-red-600 text-white flex items-center gap-2"
                >
                  <Trash2 size={18} />
                  清空所有
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
