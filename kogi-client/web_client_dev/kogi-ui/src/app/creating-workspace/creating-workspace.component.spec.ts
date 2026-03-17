import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { CreatingWorkspaceComponent } from './creating-workspace.component';

describe('CreatingWorkspaceComponent', () => {
  let component: CreatingWorkspaceComponent;
  let fixture: ComponentFixture<CreatingWorkspaceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [CreatingWorkspaceComponent, RouterTestingModule],
    }).compileComponents();

    fixture = TestBed.createComponent(CreatingWorkspaceComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
