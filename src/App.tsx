import { useState } from 'react';
import PersonnelTab from './components/PersonnelTab';
import EmailConfigTab from './components/EmailConfigTab';
import TemplateTab from './components/TemplateTab';
import ScheduleTab from './components/ScheduleTab';
import LogTab from './components/LogTab';
import { Users, Mail, FileText, Calendar, List } from 'lucide-react';
import './index.css';

type Tab = 'personnel' | 'config' | 'template' | 'schedule' | 'log';

const TABS: { key: Tab; label: string; icon: React.ReactNode }[] = [
  { key: 'personnel', label: '人员管理', icon: <Users size={16} /> },
  { key: 'config', label: '邮件配置', icon: <Mail size={16} /> },
  { key: 'template', label: '邮件模板', icon: <FileText size={16} /> },
  { key: 'schedule', label: '排班表', icon: <Calendar size={16} /> },
  { key: 'log', label: '发送日志', icon: <List size={16} /> },
];

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('personnel');

  const renderTab = () => {
    switch (activeTab) {
      case 'personnel': return <PersonnelTab />;
      case 'config': return <EmailConfigTab />;
      case 'template': return <TemplateTab />;
      case 'schedule': return <ScheduleTab />;
      case 'log': return <LogTab />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 flex flex-col">
      {/* Title Bar */}
      <div className="bg-gray-900 border-b border-gray-800 px-4 py-2 flex items-center gap-3">
        <span className="text-xl">🐂</span>
        <h1 className="text-lg font-bold">牛马人 · 值班助手</h1>
      </div>

      {/* Tab Bar */}
      <div className="flex border-b border-gray-800 bg-gray-900">
        {TABS.map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex items-center gap-2 px-4 py-2.5 text-sm transition-colors border-b-2 ${
              activeTab === tab.key
                ? 'border-amber-500 text-amber-400 bg-gray-900'
                : 'border-transparent text-gray-400 hover:text-gray-200 hover:bg-gray-800'
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-auto p-4">
        {renderTab()}
      </div>

      {/* Status Bar */}
      <div className="bg-gray-900 border-t border-gray-800 px-4 py-1.5 text-xs text-gray-500 flex items-center gap-4">
        <span className="flex items-center gap-1">
          <span className="w-2 h-2 rounded-full bg-green-500" />
          运行中
        </span>
        <span>牛马人 v0.1.0</span>
      </div>
    </div>
  );
}

export default App;
