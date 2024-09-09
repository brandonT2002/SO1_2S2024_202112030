#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/mm.h>
#include <linux/sched.h>
#include <linux/jiffies.h>
#include <linux/uaccess.h>
#include <linux/fs.h>
#include <linux/sched/signal.h>
#include <linux/slab.h>
#include <linux/binfmts.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Brandon Tejaxún");
MODULE_DESCRIPTION("Modulo para leer informacion de memoria y CPU en JSON");
MODULE_VERSION("1.0");

#define PROC_NAME "sysinfo_202112030"
#define MAX_CMDLINE_LENGTH 256

static char *get_process_cmdline(struct task_struct *task) {
    struct mm_struct *mm;
    char *cmdline, *p;
    unsigned long arg_start, arg_end;
    int len, i;

    cmdline = kmalloc(MAX_CMDLINE_LENGTH, GFP_KERNEL);
    if (!cmdline)
        return NULL;

    mm = get_task_mm(task);
    if (!mm) {
        kfree(cmdline);
        return NULL;
    }

    down_read(&mm->mmap_lock);
    arg_start = mm->arg_start;
    arg_end = mm->arg_end;
    up_read(&mm->mmap_lock);

    len = arg_end - arg_start;
    if (len > MAX_CMDLINE_LENGTH - 1)
        len = MAX_CMDLINE_LENGTH - 1;

    if (access_process_vm(task, arg_start, cmdline, len, 0) != len) {
        mmput(mm);
        kfree(cmdline);
        return NULL;
    }

    cmdline[len] = '\0';
    p = cmdline;
    for (i = 0; i < len; i++)
        if (p[i] == '\0')
            p[i] = ' ';

    mmput(mm);
    return cmdline;
}

static int sysinfo_show(struct seq_file *m, void *v)
{
    struct sysinfo si;
    struct task_struct *task;
    struct task_struct *hijos;
    int first_process = 1;
    unsigned long total_jiffies = jiffies;

    // Obtener la información de memoria
    si_meminfo(&si);
    unsigned long totalram = si.totalram * (PAGE_SIZE / 1024); // Convertir páginas a KB
    unsigned long freeram = si.freeram * (PAGE_SIZE / 1024);   // Convertir páginas a KB
    unsigned long ram_usada = totalram - freeram;

    // Imprimir información de memoria
    seq_printf(m, "  {\n");
    seq_printf(m, "\"SystemInfo\": \n");
    seq_printf(m, "\t{\n");
    seq_printf(m, "\t\t\"Total_RAM\": %lu,\n", totalram);
    seq_printf(m, "\t\t\"Free_RAM\": %lu,\n", freeram);
    seq_printf(m, "\t\t\"Used_RAM\": %lu\n", ram_usada);
    seq_printf(m, "\t},\n");

    seq_printf(m, "\"Processes\": [\n");

    // Iterar sobre los procesos
    for_each_process(task)
    {
        if (strcmp(task->comm, "containerd-shim") == 0)
        {
            unsigned long vsz = 0;
            unsigned long rss = 0;
            unsigned long mem_usage = 0;
            unsigned long cpu_usage = 0;
            char *cmdline = NULL;
            
            list_for_each_entry(hijos, &task->children, sibling)
            {
                if (hijos->mm)
                {
                    // Obtener el uso de memoria virtual y física
                    vsz = hijos->mm->total_vm << (PAGE_SHIFT - 10); // Convertir a KB
                    rss = get_mm_rss(hijos->mm) << (PAGE_SHIFT - 10); // Convertir a KB
                    mem_usage = (rss * 10000) / totalram; // Porcentaje de memoria
                }

                // Obtener el tiempo total de CPU
                unsigned long total_time = (task->utime + task->stime) + (hijos->utime + hijos->stime);
                cpu_usage = (total_time * 10000) / (total_jiffies * HZ); // Porcentaje de CPU

                cmdline = get_process_cmdline(task);

                if (!first_process)
                {
                    seq_printf(m, ",\n");
                }
                else
                {
                    first_process = 0;
                }

                seq_printf(m, "\t{\n");
                seq_printf(m, "\t\t\"PID\": %d,\n", task->pid);
                seq_printf(m, "\t\t\"Name\": \"%s\",\n", task->comm);
                seq_printf(m, "\t\t\"Cmdline\": \"%s\",\n", cmdline ? cmdline : "N/A");
                seq_printf(m, "\t\t\"MemoryUsage\": %lu.%02lu,\n", mem_usage / 100, mem_usage % 100);
                seq_printf(m, "\t\t\"CPUUsage\": %lu.%02lu\n", cpu_usage / 100, cpu_usage % 100);
                seq_printf(m, "\t}");

                if (cmdline)
                {
                    kfree(cmdline);
                }
            }
        }
    }

    seq_printf(m, "\n]\n}\n");
    return 0;
}



static int sysinfo_open(struct inode *inode, struct file *file) {
    return single_open(file, sysinfo_show, NULL);
}

static const struct proc_ops sysinfo_ops = {
    .proc_open = sysinfo_open,
    .proc_read = seq_read,
};

static int __init sysinfo_init(void) {
    proc_create(PROC_NAME, 0, NULL, &sysinfo_ops);
    printk(KERN_INFO "sysinfo_202112030 loaded\n");
    return 0;
}

static void __exit sysinfo_exit(void) {
    remove_proc_entry(PROC_NAME, NULL);
    printk(KERN_INFO "sysinfo_202112030 unloaded\n");
}

module_init(sysinfo_init);
module_exit(sysinfo_exit);
